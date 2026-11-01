// SPDX-License-Identifier: LGPL-3.0-or-later
// CableBusContainer's persisted center/side slots and GridNode's gn data.
// Registry/facade codecs and unsupported side-part implementations remain open.
use super::ae2_cable::Cable;
use super::ae2_grid::Direction;
use super::ae2_network::{Kind, Network, Pos, Snapshot};
use super::ae2_part_storage::PartStorage;
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::collections::BTreeMap;

pub const BLOCK: &str = "ae2:cable_bus";
pub const SIDES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];
pub const FACADES: [&str; 6] = [
    "facadeDown",
    "facadeUp",
    "facadeNorth",
    "facadeSouth",
    "facadeWest",
    "facadeEast",
];

#[derive(Clone, Debug)]
pub struct Part {
    pub id: String,
    pub data: Compound,
}
#[derive(Clone, Debug)]
pub struct Saved {
    pub parts: PartStorage<Part, Part, Tag>,
    pub redstone: i32,
}
fn key(value: &str) -> JavaString {
    JavaString::from(value)
}
fn part(value: Option<&Tag>) -> Result<Option<Part>, &'static str> {
    let Some(Tag::Compound(data)) = value else {
        return Ok(None);
    };
    let Some(Tag::String(id)) = data.get(&key("id")) else {
        return Err("part id missing or not a string");
    };
    let id = id.to_string().map_err(|_| "part id is not Unicode")?;
    // Full ResourceLocation and IPartItem registry recovery is not yet ported.
    if id.is_empty() {
        return Err("empty persisted part id");
    }
    Ok(Some(Part {
        id,
        data: data.clone(),
    }))
}
fn node(part: &Part) -> Option<&Compound> {
    match part.data.get(&key("gn")) {
        Some(Tag::Compound(node)) => Some(node),
        _ => None,
    }
}
/// GridNode.areTagsEqualIgnoringPlayerId treats absent, empty and p-only alike.
fn same_node_data(a: Option<&Compound>, b: Option<&Compound>) -> bool {
    super::ae2_node_data::same_ignoring_owner(a, b, &key("p"))
}
impl Saved {
    pub fn read(host: &Compound) -> Result<Self, &'static str> {
        let mut parts = PartStorage::default();
        parts.set_center(part(host.get(&key("cable")))?);
        for (i, side) in Direction::ALL.into_iter().enumerate() {
            if let Some(value) = part(host.get(&key(SIDES[i])))? {
                parts.set_part(side, Some(value));
            }
            // Keep raw facade BlockState codec payloads, never manufacture a
            // facade from a block name. Actual codec/shape support is pending.
            if let Some(tag) = host.get(&key(FACADES[i])) {
                parts.set_facade(side, Some(tag.clone()));
            }
        }
        let redstone = match host.get(&key("hasRedstone")) {
            None => 2,
            Some(Tag::Int(v)) => *v,
            _ => return Err("unsupported YesNo tag type"),
        };
        if !(0..=2).contains(&redstone) {
            return Err("invalid YesNo ordinal");
        }
        Ok(Self { parts, redstone })
    }
    pub fn center(&self) -> Result<Option<Cable>, &'static str> {
        self.parts
            .center()
            .map(|p| Cable::from_item(&p.id).ok_or("unsupported center part"))
            .transpose()
    }
    pub fn center_snapshot(&self, pos: Pos) -> Result<Option<Snapshot>, &'static str> {
        let Some(cable) = self.center()? else {
            return Ok(None);
        };
        // Attached parts have their own grid nodes/services. Activating only
        // the center would silently omit them; keep this host inactive until
        // those parts are implemented. The complete raw NBT remains untouched.
        if Direction::ALL
            .into_iter()
            .any(|side| self.parts.part(side).is_some())
        {
            return Err("side-part implementation unavailable");
        }
        let center = self.parts.center().unwrap();
        // EnergyService.addNode checks contains(e, TAG_DOUBLE), not the
        // generic numeric getter. Integer/float fields do not restore energy.
        let saved_buffer = match node(center).and_then(|n| n.get(&key("e"))) {
            Some(Tag::Double(value)) => f64::from_bits(*value),
            _ => 0.0,
        };
        if !saved_buffer.is_finite() || saved_buffer < 0.0 {
            return Err("unsupported node energy");
        }
        Ok(Some(Snapshot {
            pos,
            kind: Kind::Cable(cable),
            front: Direction::North,
            exposed: 63,
            stored: 0.0,
            saved_buffer,
        }))
    }
}
#[derive(Clone, Debug)]
struct Observed {
    part: Part,
}
#[derive(Default)]
pub struct Hosts {
    observed: BTreeMap<Pos, Observed>,
}
impl Hosts {
    /// Source loadPart replaces the part when item identity differs, including
    /// externally edited color IDs. Actual painting will update the existing
    /// node separately; it is not the NBT loadPart path.
    pub fn observe(
        &mut self,
        pos: Pos,
        host: &Compound,
    ) -> Result<(Option<Snapshot>, bool), &'static str> {
        let saved = Saved::read(host)?;
        let snapshot = saved.center_snapshot(pos)?;
        let Some(part) = saved.parts.center() else {
            return Ok((None, self.observed.remove(&pos).is_some()));
        };
        let replaced = self.observed.get(&pos).is_some_and(|old| {
            old.part.id != part.id || !same_node_data(node(&old.part), node(part))
        });
        self.observed.insert(pos, Observed { part: part.clone() });
        Ok((snapshot, replaced))
    }
    pub fn snapshot(
        &mut self,
        pos: Pos,
        host: &Compound,
        network: &mut Network,
    ) -> Result<Option<Snapshot>, &'static str> {
        let (snapshot, replaced) = self.observe(pos, host)?;
        if replaced {
            network.forget_node(pos);
        }
        Ok(snapshot)
    }
    pub fn retain(&mut self, positions: &std::collections::BTreeSet<Pos>) {
        self.observed.retain(|pos, _| positions.contains(pos));
    }
    /// Return a new gn only if the live center still equals the observed one.
    /// Preserve host/part/facade fields and unknown service data; this is not a
    /// complete Grid.saveNodeData implementation. p is always an Int on save.
    pub fn persist(&mut self, pos: Pos, host: &mut Compound, cable: Cable, share: f64) -> bool {
        if !share.is_finite() || share < 0.0 {
            return false;
        }
        let Some(old) = self.observed.get_mut(&pos) else {
            return false;
        };
        let Ok(Some(current)) = part(host.get(&key("cable"))) else {
            return false;
        };
        if Cable::from_item(&current.id) != Some(cable)
            || current.id != old.part.id
            || current.data != old.part.data
        {
            return false;
        }
        let mut data = current.data;
        let mut gn = node(&old.part).cloned().unwrap_or_default();
        let owner = match gn.get(&key("p")) {
            Some(Tag::Int(v)) => *v,
            _ => -1,
        };
        gn.insert(key("p"), Tag::Int(owner));
        if share > 0.0 {
            gn.insert(key("e"), Tag::Double(share.to_bits()));
        } else {
            gn.remove(&key("e"));
        }
        data.insert(key("gn"), Tag::Compound(gn));
        let changed = data != old.part.data;
        old.part.data = data.clone();
        if changed {
            host.insert(key("cable"), Tag::Compound(data));
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::super::ae2_cable::{CableType, Color};
    use super::*;
    fn host(color: Color, energy: f64) -> Compound {
        let cable = Cable {
            kind: CableType::Smart,
            color,
        };
        Compound::from([(
            key("cable"),
            Tag::compound([
                ("id", Tag::String(cable.item().unwrap().as_str().into())),
                (
                    "gn",
                    Tag::compound([("p", Tag::Int(12)), ("e", Tag::Double(energy.to_bits()))]),
                ),
            ]),
        )])
    }
    fn center(host: &mut Compound) -> &mut Compound {
        let Tag::Compound(v) = host.get_mut(&key("cable")).unwrap() else {
            unreachable!()
        };
        v
    }
    #[test]
    fn all_source_centers_read_only_gn_not_full_block_proxy_and_keep_facades_and_side_data() {
        for kind in CableType::VALID {
            for color in Color::ALL {
                let cable = Cable { kind, color };
                let mut h = host(color, 42.0);
                center(&mut h).insert(
                    key("id"),
                    Tag::String(cable.item().unwrap().as_str().into()),
                );
                h.insert(
                    key("proxy"),
                    Tag::compound([("e", Tag::Double(999.0_f64.to_bits()))]),
                );
                h.insert(
                    key("facadeEast"),
                    Tag::compound([("Name", Tag::String("minecraft:stone".into()))]),
                );
                let saved = Saved::read(&h).unwrap();
                let s = saved.center_snapshot([1, 2, 3]).unwrap().unwrap();
                assert_eq!(s.kind, Kind::Cable(cable));
                assert_eq!(s.saved_buffer, 42.0);
                assert_eq!(saved.redstone, 2);
                assert!(saved.parts.facade(Direction::East).is_some());
                h.insert(
                    key("north"),
                    Tag::compound([
                        ("id", Tag::String("ae2:terminal".into())),
                        ("custom", Tag::Long(123)),
                    ]),
                );
                let saved = Saved::read(&h).unwrap();
                assert!(saved.center_snapshot([1, 2, 3]).is_err());
                assert!(
                    saved
                        .parts
                        .part(Direction::North)
                        .unwrap()
                        .data
                        .contains_key(&key("custom"))
                );
            }
        }
    }
    #[test]
    fn owner_only_load_preserves_identity_but_external_node_data_or_item_replacement_does_not() {
        let mut hosts = Hosts::default();
        let mut h = host(Color::Red, 20.0);
        let p = [1, 2, 3];
        assert!(!hosts.observe(p, &h).unwrap().1);
        let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(key("p"), Tag::Int(99));
        assert!(!hosts.observe(p, &h).unwrap().1);
        let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(key("e"), Tag::Double(30.0_f64.to_bits()));
        assert!(hosts.observe(p, &h).unwrap().1);
        center(&mut h).insert(key("id"), Tag::String("ae2:blue_smart_cable".into()));
        assert!(hosts.observe(p, &h).unwrap().1);
        assert!(same_node_data(
            None,
            Some(&Compound::from([(key("p"), Tag::Int(1))]))
        ));
    }
    #[test]
    fn saves_update_only_center_gn_and_never_overwrite_a_new_external_part_snapshot() {
        let mut hosts = Hosts::default();
        let mut h = host(Color::Red, 20.0);
        let p = [0, 0, 0];
        let cable = Cable {
            kind: CableType::Smart,
            color: Color::Red,
        };
        h.insert(key("unknownHost"), Tag::Long(71));
        center(&mut h).insert(key("customName"), Tag::String("saved".into()));
        hosts.observe(p, &h).unwrap();
        assert!(hosts.persist(p, &mut h, cable, 5.0));
        assert!(!hosts.observe(p, &h).unwrap().1);
        assert!(!hosts.persist(p, &mut h, cable, 5.0));
        assert_eq!(
            Saved::read(&h)
                .unwrap()
                .center_snapshot(p)
                .unwrap()
                .unwrap()
                .saved_buffer,
            5.0
        );
        assert_eq!(h.get(&key("unknownHost")), Some(&Tag::Long(71)));
        assert_eq!(
            center(&mut h).get(&key("customName")),
            Some(&Tag::String("saved".into()))
        );
        center(&mut h).insert(key("external"), Tag::Int(4));
        let external = h.clone();
        assert!(!hosts.persist(p, &mut h, cable, 0.0));
        assert_eq!(h, external);
        hosts.observe(p, &h).unwrap();
        assert!(hosts.persist(p, &mut h, cable, 0.0));
        assert_eq!(
            Saved::read(&h)
                .unwrap()
                .center_snapshot(p)
                .unwrap()
                .unwrap()
                .saved_buffer,
            0.0
        );
    }
    #[test]
    fn typed_nbt_restart_keeps_host_part_facade_and_service_data_with_exact_buffer() {
        use horizon_world::nbt::{self, Limits};
        let pos = [3, 4, 5];
        let mut h = host(Color::Cyan, 23.5);
        h.insert(
            key("opaqueHost"),
            Tag::LongArray(vec![i64::MIN, 0, i64::MAX]),
        );
        h.insert(
            key("facadeEast"),
            Tag::compound([("Name", Tag::String("minecraft:stone".into()))]),
        );
        center(&mut h).insert(key("opaquePart"), Tag::ByteArray(vec![0, 127, 255]));
        let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(
            key("opaqueService"),
            Tag::IntArray(vec![i32::MIN, 42, i32::MAX]),
        );
        let mut hosts = Hosts::default();
        let cable = Saved::read(&h).unwrap().center().unwrap().unwrap();
        hosts.observe(pos, &h).unwrap();
        assert!(hosts.persist(pos, &mut h, cable, 7.25));
        let tag = Tag::Compound(h.clone());
        let bytes = nbt::write_network(&tag, Limits::saved()).unwrap();
        let (loaded, used) = nbt::read_network(&bytes, Limits::saved()).unwrap();
        assert_eq!(used, bytes.len());
        assert_eq!(loaded, tag);
        let Tag::Compound(loaded) = loaded else {
            unreachable!()
        };
        let mut restarted = Hosts::default();
        let (snapshot, replaced) = restarted.observe(pos, &loaded).unwrap();
        assert!(!replaced);
        assert_eq!(snapshot.unwrap().saved_buffer, 7.25);
        assert_eq!(loaded.get(&key("opaqueHost")), h.get(&key("opaqueHost")));
        assert!(
            Saved::read(&loaded)
                .unwrap()
                .parts
                .facade(Direction::East)
                .is_some()
        );
    }
    #[test]
    fn malformed_or_unsupported_host_refusal_preserves_the_entire_original_nbt() {
        let mut hosts = Hosts::default();
        for energy in [f64::NAN, f64::INFINITY, -1.0] {
            let h = host(Color::Red, energy);
            let original = h.clone();
            assert!(hosts.observe([0, 0, 0], &h).is_err());
            assert_eq!(h, original);
        }
        let mut h = host(Color::Red, 5.0);
        h.insert(key("hasRedstone"), Tag::Int(3));
        assert!(hosts.observe([0, 0, 0], &h).is_err());
        h.insert(key("hasRedstone"), Tag::Byte(1));
        assert!(hosts.observe([0, 0, 0], &h).is_err());
        h.remove(&key("hasRedstone"));
        center(&mut h).insert(key("id"), Tag::String("other:missing_part".into()));
        let original = h.clone();
        assert!(hosts.observe([0, 0, 0], &h).is_err());
        assert_eq!(h, original);
    }
    #[test]
    fn live_graph_buffer_is_not_refilled_by_our_save_and_external_gn_reload_replaces_the_node() {
        let pos = [1, 1, 1];
        let mut h = host(Color::Red, 100.0);
        let mut hosts = Hosts::default();
        let mut network = Network::default();
        let snapshots = hosts
            .snapshot(pos, &h, &mut network)
            .unwrap()
            .into_iter()
            .collect();
        network.sync(snapshots);
        let first = network.node_service(pos).unwrap().1;
        // The source's grid-local buffer is 25 AE per node, not item storage.
        assert_eq!(network.extract(pos, 1000.0, 1.0, false), 25.0);
        assert_eq!(network.extract(pos, 10.0, 1.0, true), 10.0);
        for _ in 0..3 {
            let state = network.end_tick().remove(0);
            let Kind::Cable(cable) = state.kind else {
                unreachable!()
            };
            hosts.persist(pos, &mut h, cable, state.buffer_share);
            let snapshots = hosts
                .snapshot(pos, &h, &mut network)
                .unwrap()
                .into_iter()
                .collect();
            network.sync(snapshots);
            assert_eq!(network.node_service(pos).unwrap().1, first);
            assert_eq!(network.extract(pos, 1000.0, 1.0, false), 15.0);
        }
        let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(key("p"), Tag::Int(99));
        let snapshots = hosts
            .snapshot(pos, &h, &mut network)
            .unwrap()
            .into_iter()
            .collect();
        network.sync(snapshots);
        assert_eq!(network.node_service(pos).unwrap().1, first);
        assert_eq!(network.extract(pos, 1000.0, 1.0, false), 15.0);
        let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(key("e"), Tag::Double(55.0_f64.to_bits()));
        let snapshots = hosts
            .snapshot(pos, &h, &mut network)
            .unwrap()
            .into_iter()
            .collect();
        network.sync(snapshots);
        assert_ne!(network.node_service(pos).unwrap().1, first);
        assert_eq!(network.extract(pos, 1000.0, 1.0, false), 25.0);
    }
    #[test]
    fn only_double_typed_saved_node_energy_is_restored_as_in_energy_service_add_node() {
        let mut h = host(Color::Red, 5.0);
        for value in [
            Tag::Byte(5),
            Tag::Short(5),
            Tag::Int(5),
            Tag::Long(5),
            Tag::Float(5.0_f32.to_bits()),
            Tag::String("5".into()),
        ] {
            let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
                unreachable!()
            };
            gn.insert(key("e"), value);
            assert_eq!(
                Saved::read(&h)
                    .unwrap()
                    .center_snapshot([0, 0, 0])
                    .unwrap()
                    .unwrap()
                    .saved_buffer,
                0.0
            );
        }
    }
}
