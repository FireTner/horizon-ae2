// SPDX-License-Identifier: LGPL-3.0-or-later
// EnergyCellBlockEntity import/exportSettings(DISMANTLE_ITEM), limited to its
// stored-energy component. Base naming/ownership/settings APIs remain separate.
use super::{Game, GameLevel, ae2_energy, ae2_energy_cell_glue, ae2_grid_glue, ae2_network::Kind};
use horizon_client::{
    containers::{BlockEntity, ModBlockEntity},
    items::ItemStack,
};
use horizon_world::nbt::{Compound, JavaString, Tag};

fn saved(be: &ModBlockEntity) -> f64 {
    match be.nbt.get(&JavaString::from("internalCurrentPower")) {
        Some(Tag::Double(bits)) if f64::from_bits(*bits).is_finite() => f64::from_bits(*bits),
        _ => 0.0,
    }
}
fn flush_entity(dimension: &str, pos: [i32; 3], be: &mut ModBlockEntity) -> bool {
    if ae2_energy_cell_glue::capacity(&be.id).is_none() {
        return false;
    }
    let Some(amount) =
        ae2_grid_glue::cell_stored(dimension, pos, Kind::from_block(&be.id).unwrap(), saved(be))
    else {
        return false;
    };
    be.nbt
        .insert("internalCurrentPower".into(), Tag::Double(amount.to_bits()));
    true
}
pub(super) fn flush(game: &mut Game<'_>, pos: [i32; 3]) {
    let actual = game.native_block(pos);
    if let Some(BlockEntity::Mod(be)) = game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2]))
        && be.id == actual
        && flush_entity(game.dim.id(), pos, be)
    {
        game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
    }
}
pub(super) fn flush_level(level: &mut GameLevel<'_>, pos: [i32; 3]) {
    let actual = level
        .facts
        .get(level.world.block(pos[0], pos[1], pos[2]))
        .name
        .clone();
    if let Some(BlockEntity::Mod(be)) = level
        .world
        .block_entities
        .get_mut(&(pos[0], pos[1], pos[2]))
        && be.id == &*actual
        && flush_entity(
            match level.dimension {
                horizon_world::level::Dimension::Overworld => "minecraft:overworld",
                horizon_world::level::Dimension::Nether => "minecraft:the_nether",
                horizon_world::level::Dimension::End => "minecraft:the_end",
            },
            pos,
            be,
        )
    {
        level.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
    }
}
pub(super) fn placed(game: &mut Game<'_>, stack: &ItemStack, pos: [i32; 3]) {
    let Some(capacity) = ae2_energy_cell_glue::capacity(&stack.item) else {
        return;
    };
    if game.native_block(pos) != &*stack.item {
        return;
    }
    let Ok(Some(item)) = super::ae2_item_energy_glue::load(stack) else {
        return;
    };
    let amount = item.energy.current();
    if !amount.is_finite() {
        return;
    } // Existing native finite-power malformed gate.
    let amount = if amount < ae2_energy::MIN_AMOUNT {
        0.0
    } else {
        amount.clamp(0.0, capacity)
    };
    if let Some(BlockEntity::Mod(be)) = game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2]))
        && be.id == &*stack.item
    {
        be.nbt
            .insert("internalCurrentPower".into(), Tag::Double(amount.to_bits()));
        game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
        ae2_grid_glue::cell_placed(game.dim.id(), pos);
    }
}

pub(super) fn export(entity: &BlockEntity, block: &str) -> Option<Compound> {
    let BlockEntity::Mod(be) = entity else {
        return None;
    };
    if be.id != block {
        return None;
    }
    let capacity = ae2_energy_cell_glue::capacity(block)?;
    // A source loadTag/importSettings has already applied setStored before
    // exportSettings reads it. Native /data may still hold the input NBT here.
    let current = saved(be);
    let current = if current < ae2_energy::MIN_AMOUNT {
        0.0
    } else {
        current.min(capacity)
    };
    if current <= 0.0 {
        return None;
    }
    Some([("ae2:stored_energy".into(), Tag::Double(current.to_bits()))].into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn energy_export_omits_zero_and_proxy_and_attaches_to_only_the_matching_drop_once() {
        let mut be = ModBlockEntity {
            id: "ae2:energy_cell".into(),
            ..Default::default()
        };
        be.nbt.insert(
            "proxy".into(),
            Tag::compound([("e", Tag::Double(1000.0f64.to_bits()))]),
        );
        assert!(export(&BlockEntity::Mod(Box::new(be.clone())), "ae2:energy_cell").is_none());
        be.nbt.insert(
            "internalCurrentPower".into(),
            Tag::Double(0.0000005f64.to_bits()),
        );
        assert!(export(&BlockEntity::Mod(Box::new(be.clone())), "ae2:energy_cell").is_none());
        be.nbt.insert(
            "internalCurrentPower".into(),
            Tag::Double(200001.0f64.to_bits()),
        );
        assert_eq!(
            export(&BlockEntity::Mod(Box::new(be.clone())), "ae2:energy_cell").unwrap()
                [&JavaString::from("ae2:stored_energy")],
            Tag::Double(200000.0f64.to_bits())
        );
        be.nbt.insert(
            "internalCurrentPower".into(),
            Tag::Double(12345.0f64.to_bits()),
        );
        let be = BlockEntity::Mod(Box::new(be));
        let mut pending = super::super::spill::Pending::default();
        pending.record([1, 2, 3], &be, "ae2:energy_cell", None);
        let mut foreign = ItemStack::new("minecraft:stone", 1);
        pending.attach([1, 2, 3], &mut foreign);
        assert!(foreign.extra.is_none());
        let mut own = ItemStack::new("ae2:energy_cell", 1);
        pending.attach([1, 2, 3], &mut own);
        assert_eq!(
            own.data().unwrap().other("ae2:stored_energy"),
            Some(&Tag::Double(12345.0f64.to_bits()))
        );
        assert!(own.data().unwrap().other("proxy").is_none());
        let mut second = ItemStack::new("ae2:energy_cell", 1);
        pending.attach([1, 2, 3], &mut second);
        assert!(second.extra.is_none());
    }
}
