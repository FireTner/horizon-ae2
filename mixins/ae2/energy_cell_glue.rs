// SPDX-License-Identifier: LGPL-3.0-or-later
// EnergyCellBlock / EnergyCellBlockEntity: fullness and deferred comparators.
use super::{Game, ae2_energy, ae2_network::Kind};
use horizon_client::containers::BlockEntity;
use horizon_world::nbt::{JavaString, Tag};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(super) struct Displays {
    seen: BTreeMap<[i32; 3], (Kind, u64)>,
}

pub(super) fn capacity(id: &str) -> Option<f64> {
    match Kind::from_block(id)? {
        Kind::EnergyCell => Some(Kind::EnergyCell.capacity()),
        Kind::DenseEnergyCell => Some(Kind::DenseEnergyCell.capacity()),
        _ => None, // CreativeEnergyCellBlock has neither fullness nor comparator.
    }
}

pub(super) fn signal(id: &str, nbt: &horizon_world::nbt::Compound) -> Option<i32> {
    let maximum = capacity(id)?;
    let current = match nbt.get(&JavaString::from("internalCurrentPower")) {
        Some(Tag::Double(value))
            if f64::from_bits(*value).is_finite()
                && f64::from_bits(*value) >= ae2_energy::MIN_AMOUNT =>
        {
            f64::from_bits(*value).min(maximum)
        }
        _ => 0.0,
    };
    Some(ae2_energy::cell_signal(current, maximum))
}

impl Displays {
    pub(super) fn collect(&mut self, game: &mut Game<'_>) -> Vec<([i32; 3], u8, bool)> {
        let mut live = BTreeSet::new();
        let mut updates = Vec::new();
        let actual: BTreeMap<_, _> = game
            .world
            .block_entities
            .iter()
            .filter_map(|(&(x, y, z), entity)| {
                (matches!(entity, BlockEntity::Mod(be) if capacity(&be.id).is_some())
                    && game.world.loaded(x, z))
                .then(|| ([x, y, z], game.native_block([x, y, z])))
            })
            .collect();
        for ([x, y, z], block) in actual {
            let Some(BlockEntity::Mod(be)) = game.world.block_entities.get_mut(&(x, y, z)) else {
                continue;
            };
            let Some(maximum) = capacity(&be.id) else {
                continue;
            };
            if block != be.id {
                continue;
            }
            let pos = [x, y, z];
            live.insert(pos);
            let Some(Tag::Double(value)) = be.nbt.get(&JavaString::from("internalCurrentPower"))
            else {
                continue;
            };
            let amount = f64::from_bits(*value);
            let sample = (Kind::from_block(&be.id).unwrap(), amount.to_bits());
            let changed = self.seen.get(&pos) != Some(&sample);
            let pending = matches!(be.nbt.get(&JavaString::from("neighborChangePending")), Some(Tag::Byte(value)) if *value != 0);
            // The grid may drain loaded storage in a non-ticking chunk. Keep
            // its deferred notification until the chunk can tick again.
            if changed {
                be.nbt.insert("neighborChangePending".into(), Tag::Byte(1));
                game.world.modified.insert((x >> 4, z >> 4));
            }
            self.seen.insert(pos, sample);
            if !game.tickets.levels().block_ticking_at(x, z) {
                continue;
            }
            be.nbt.insert("neighborChangePending".into(), Tag::Byte(0));
            if pending {
                game.world.modified.insert((x >> 4, z >> 4));
            }
            updates.push((
                pos,
                ae2_energy::display_level(amount / maximum),
                changed || pending,
            ));
        }
        self.seen.retain(|pos, _| live.contains(pos));
        updates
    }
}

pub(super) fn apply(game: &mut Game<'_>, updates: Vec<([i32; 3], u8, bool)>) {
    for (pos, fullness, notify) in updates {
        // Source updateStateForPowerLevel uses setBlockAndUpdate, rather
        // than Controller's UPDATE_CLIENTS-only spatial-move policy.
        game.native_set_block_property(pos, "fullness", &fullness.to_string());
        if notify {
            game.container_changed((pos[0], pos[1], pos[2]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_world::nbt::Compound;

    #[test]
    fn native_cell_signal_reads_local_storage_not_proxy_and_leaves_foreign_hosts_alone() {
        let mut nbt = Compound::new();
        let mut proxy = Compound::new();
        proxy.insert("e".into(), Tag::Double(1000000.0f64.to_bits()));
        nbt.insert("proxy".into(), Tag::Compound(proxy));
        assert_eq!(signal("ae2:energy_cell", &nbt), Some(0));
        nbt.insert(
            "internalCurrentPower".into(),
            Tag::Double(100000.0f64.to_bits()),
        );
        assert_eq!(signal("ae2:energy_cell", &nbt), Some(8));
        assert_eq!(signal("ae2:dense_energy_cell", &nbt), Some(1));
        assert_eq!(signal("ae2:creative_energy_cell", &nbt), None);
        assert_eq!(signal("ae2:controller", &nbt), None);
        assert_eq!(signal("minecraft:barrel", &nbt), None);
    }
}
