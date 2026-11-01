// SPDX-License-Identifier: LGPL-3.0-or-later
// AEBaseInvBlockEntity: Charger original inv.item0, mirrored in native Items.
use super::Game;
use horizon_client::{
    containers::{BlockEntity, ModBlockEntity},
    items::ItemStack,
};
#[cfg(test)]
use horizon_world::nbt::Compound;
use horizon_world::nbt::{JavaString, Tag};
use std::collections::{BTreeMap, BTreeSet};
type Pos = [i32; 3];
const CHARGER: &str = "ae2:charger";

fn decode(tag: &Tag) -> Result<Option<ItemStack>, &'static str> {
    let mut items = super::ae2_inventory_codec::read_slots(tag, 1)?;
    Ok(items.remove(0))
}
fn encode(stack: Option<&ItemStack>, previous: Option<&Tag>) -> Tag {
    super::ae2_inventory_codec::write_slots(std::iter::once(stack), previous)
}
pub(super) fn usable(be: &ModBlockEntity) -> bool {
    be.nbt
        .get(&JavaString::from("inv"))
        .is_none_or(|inv| decode(inv).is_ok())
}
pub(super) fn persist(be: &mut ModBlockEntity) {
    if !usable(be) {
        return;
    }
    let inv = encode(
        be.items.first().and_then(Option::as_ref),
        be.nbt.get(&JavaString::from("inv")),
    );
    be.nbt.insert(JavaString::from("inv"), inv);
}
#[derive(Default)]
struct Echo {
    observed: bool,
    inv: Option<Tag>,
    item: Option<ItemStack>,
    blocked: bool,
}
impl Echo {
    fn sync(&mut self, be: &mut ModBlockEntity) {
        let inv = be.nbt.get(&JavaString::from("inv")).cloned();
        let native = be.items.first().cloned().flatten();
        if !self.observed || self.inv != inv || self.item != native || self.blocked {
            let chosen = super::ae2_inventory_codec::choose_slots(
                inv.as_ref(),
                self.inv.as_ref(),
                std::slice::from_ref(&native),
                self.observed,
                self.blocked,
                1,
            )
            .map(|mut items| items.remove(0));
            match chosen {
                Ok(item) => {
                    if be.items.is_empty() {
                        be.items.push(None);
                    }
                    be.items[0] = item;
                    self.blocked = false;
                    persist(be);
                }
                Err(_) => self.blocked = true, // Preserve raw NBT and the native echo.
            }
            self.inv = be.nbt.get(&JavaString::from("inv")).cloned();
            self.item = be.items.first().cloned().flatten();
            self.observed = true;
        }
    }
}
#[derive(Default)]
pub(super) struct Inventories {
    echoes: BTreeMap<Pos, Echo>,
}
impl Inventories {
    pub(super) fn sync(&mut self, game: &mut Game<'_>) {
        let positions: Vec<_> = game
            .world
            .block_entities
            .iter()
            .filter_map(|(&(x, y, z), be)| {
                matches!(be,BlockEntity::Mod(be) if be.id==CHARGER)
                    .then_some([x, y, z])
                    .filter(|_| game.world.loaded(x, z) && game.native_block([x, y, z]) == CHARGER)
            })
            .collect();
        let present: BTreeSet<_> = positions.iter().copied().collect();
        self.echoes.retain(|pos, _| present.contains(pos));
        for pos in positions {
            let Some(BlockEntity::Mod(be)) =
                game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2]))
            else {
                continue;
            };
            let before = (be.nbt.clone(), be.items.clone());
            self.echoes.entry(pos).or_default().sync(be);
            if before != (be.nbt.clone(), be.items.clone()) {
                game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn entity() -> ModBlockEntity {
        ModBlockEntity {
            id: CHARGER.into(),
            nbt: Compound::new(),
            items: vec![None],
            data: vec![],
        }
    }
    #[test]
    fn source_original_inv_loads_and_live_native_energy_updates_never_restore_stale_source_echo() {
        let mut be = entity();
        let mut item = ItemStack::new("ae2:portable_item_cell_1k", 1);
        item.data_mut()
            .set_other("ae2:stored_energy", Some(Tag::Double(1234.0f64.to_bits())));
        be.nbt.insert("inv".into(), encode(Some(&item), None));
        let mut echo = Echo::default();
        echo.sync(&mut be);
        assert_eq!(be.items[0], Some(item));
        be.items[0]
            .as_mut()
            .unwrap()
            .data_mut()
            .set_other("ae2:stored_energy", Some(Tag::Double(2011.0f64.to_bits())));
        echo.sync(&mut be);
        let saved = decode(be.nbt.get(&JavaString::from("inv")).unwrap()).unwrap();
        assert_eq!(saved, be.items[0]);
        echo.sync(&mut be);
        assert_eq!(be.items[0], saved);
        // An explicit original-format update wins over the previous native mirror.
        be.nbt.insert(
            "inv".into(),
            encode(Some(&ItemStack::new("ae2:charged_staff", 1)), None),
        );
        echo.sync(&mut be);
        assert_eq!(&*be.items[0].as_ref().unwrap().item, "ae2:charged_staff");
    }
    #[test]
    fn malformed_original_inventory_is_retained_and_native_mutations_cannot_unblock_it() {
        let mut be = entity();
        let bad = Tag::compound([(
            "item0",
            Tag::compound([
                ("id", Tag::String("ae2:charged_staff".into())),
                ("count", Tag::Int(0)),
            ]),
        )]);
        be.nbt.insert("inv".into(), bad.clone());
        let mut echo = Echo::default();
        echo.sync(&mut be);
        assert!(echo.blocked);
        assert!(!usable(&be));
        be.items[0] = Some(ItemStack::new("minecraft:stone", 64));
        echo.sync(&mut be);
        persist(&mut be);
        assert!(echo.blocked);
        assert_eq!(be.nbt.get(&JavaString::from("inv")), Some(&bad));
        be.nbt.insert("inv".into(), encode(None, None));
        echo.sync(&mut be);
        assert!(!echo.blocked);
        assert_eq!(be.items[0], None);
    }
    #[test]
    fn legacy_native_migration_and_extraction_write_source_empty_compound_preserving_other_nbt() {
        let mut be = entity();
        be.items[0] = Some(ItemStack::new("ae2:charged_staff", 1));
        be.nbt.insert("test:data".into(), Tag::Long(77));
        let mut echo = Echo::default();
        echo.sync(&mut be);
        assert!(be.nbt.contains_key(&JavaString::from("inv")));
        be.items[0] = None;
        persist(&mut be);
        echo.sync(&mut be);
        assert_eq!(
            decode(be.nbt.get(&JavaString::from("inv")).unwrap()).unwrap(),
            None
        );
        assert_eq!(
            be.nbt.get(&JavaString::from("test:data")),
            Some(&Tag::Long(77))
        );
    }
    #[test]
    fn source_count_default_and_overstack_are_preserved_without_native_count_clamping() {
        let encoded = encode(Some(&ItemStack::new("ae2:charged_staff", 1)), None);
        assert!(encoded.get("item0").unwrap().get("count").is_none());
        let mut inv = Tag::compound([(
            "item0",
            Tag::compound([
                ("id", Tag::String("ae2:charged_staff".into())),
                ("count", Tag::Int(2)),
            ]),
        )]);
        assert_eq!(decode(&inv).unwrap().unwrap().count, 2);
        if let Tag::Compound(ref mut map) = inv {
            map.insert(
                "item0".into(),
                Tag::compound([
                    ("id", Tag::String("ae2:charged_staff".into())),
                    ("Count", Tag::Int(7)),
                ]),
            );
        }
        assert_eq!(decode(&inv).unwrap().unwrap().count, 1);
        for bad in [Tag::Int(1), Tag::compound([("item0", Tag::Int(1))])] {
            assert!(decode(&bad).is_err());
        }
    }
}
