// SPDX-License-Identifier: LGPL-3.0-or-later
// PartPlacement + CableBusBlock.getDrops for implemented central cable parts.
use super::{
    Game,
    ae2_cable::Cable,
    ae2_cable_bus as bus,
    ae2_grid::Direction,
    ae2_part_placement::{self as part_placement, CenterSlot},
    ae2_player_registry::Players,
};
use horizon_client::{containers::BlockEntity, items::ItemStack, save};
use horizon_world::{
    fluid::{FluidKind, FluidState},
    nbt::{Compound, JavaString, Tag},
};
fn key(s: &str) -> JavaString {
    s.into()
}
fn owner(game: &Game<'_>) -> Result<i32, String> {
    let Some(player) = game.native_acting_player() else {
        return Ok(-1);
    };
    // A world-wide registry, shared by all dimensions, in the original SavedData
    // format. Do not substitute a process-local UUID hash for an ME player ID.
    owner_at(&game.world_root.join("data/ae2_players.dat"), player.uuid)
}
fn owner_at(path: &std::path::Path, uuid: u128) -> Result<i32, String> {
    let mut root = if path.exists() {
        save::read_dat(&path).ok_or("cannot read ae2_players.dat")?
    } else {
        Tag::compound([("data", Tag::Compound(Compound::new()))])
    };
    let Tag::Compound(root_fields) = &mut root else {
        return Err("invalid player registry root".into());
    };
    let Some(Tag::Compound(data)) = root_fields.get_mut(&key("data")) else {
        return Err("invalid player registry data".into());
    };
    let ids = match data.get(&key("playerIds")) {
        Some(Tag::IntArray(v)) => v.as_slice(),
        _ => &[],
    };
    let profiles = match data.get(&key("profileIds")) {
        Some(Tag::LongArray(v)) => v.as_slice(),
        _ => &[],
    };
    let mut players = Players::load(ids, profiles)?;
    let (id, changed) = players.id(uuid)?;
    if changed {
        let (ids, profiles) = players.save();
        data.insert(key("playerIds"), Tag::IntArray(ids));
        data.insert(key("profileIds"), Tag::LongArray(profiles));
        save::write_dat(&path, root).map_err(|e| e.to_string())?;
    }
    Ok(id)
}
fn slot(game: &Game<'_>, pos: [i32; 3]) -> CenterSlot {
    let state = game.world.block(pos[0], pos[1], pos[2]);
    if !game.world.loaded(pos[0], pos[2]) {
        return CenterSlot::NoHost {
            replaceable: false,
            may_interact: false,
        };
    }
    if game.native_block(pos) == bus::BLOCK {
        if let Some(BlockEntity::Mod(be)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
            && be.id == bus::BLOCK
            && bus::Saved::read(&be.nbt).is_ok()
        {
            // Preserve unsupported raw side/facade data even on rollback.
            // Complete registry-aware admission/collision is separate work.
            let compatible = bus::SIDES
                .iter()
                .chain(bus::FACADES.iter())
                .all(|name| !be.nbt.contains_key(&key(name)));
            return CenterSlot::Host {
                occupied: be.nbt.contains_key(&key("cable")),
                compatible,
            };
        }
        return CenterSlot::Host {
            occupied: true,
            compatible: false,
        };
    }
    CenterSlot::NoHost {
        replaceable: game.res.facts().get(state).replaceable,
        may_interact: !game.spectator(),
    }
}
fn fluid_water(game: &Game<'_>, pos: [i32; 3]) -> bool {
    game.pack
        .assets
        .registry
        .state(game.world.block(pos[0], pos[1], pos[2]))
        .and_then(|state| FluidState::from_block(state).ok())
        .is_some_and(|fluid| fluid.kind() == FluidKind::Water)
}
pub(super) fn use_on(
    game: &mut Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    if *claimed || Cable::from_item(&stack.item).is_none() || stack.count == 0 {
        return;
    }
    // A failed PartItem.useOn is FAIL too: never let a template item place a
    // different block or fall through to an unrelated use action.
    *claimed = true;
    if game.spectator() || (!game.may_build() && !game.can_place_on_in_adventure(pos)) {
        return;
    }
    let Some(face) = Direction::ALL.into_iter().find(|d| d.offset() == face) else {
        return;
    };
    let ctrl = game.keys.contains(&winit::keyboard::KeyCode::ControlLeft)
        || game.keys.contains(&winit::keyboard::KeyCode::ControlRight);
    let Some(target) = part_placement::center_target(pos, face, ctrl, |p| slot(game, p)) else {
        return;
    };
    let pos = target.pos;
    let was_host = game.native_block(pos) == bus::BLOCK;
    let wet = fluid_water(game, pos);
    let Some(mut state) = game.pack.assets.registry.default_state(bus::BLOCK) else {
        return;
    };
    if wet {
        state = game
            .with_props(state, &[("waterlogged", "true")])
            .unwrap_or(state);
    }
    if !was_host {
        let dimension = game.dim.id().to_owned();
        if !game.native_set_block(&dimension, pos, state, 3) {
            return;
        }
    }
    let owner = match owner(game) {
        Ok(id) => id,
        Err(error) => {
            log::warn!("AE2 cable placement: {error}");
            cleanup_empty(game, pos, wet);
            return;
        }
    };
    let mut part = Compound::new();
    part.insert(key("id"), Tag::String(stack.item.as_ref().into()));
    part.insert(key("gn"), Tag::compound([("p", Tag::Int(owner))]));
    if let Some(name) = stack.data().and_then(|d| d.custom_name.as_ref()) {
        let json = horizon_client::scoreboard::tag_to_json(name).to_string();
        part.insert(key("customName"), Tag::String(json.as_str().into()));
    }
    let added = game
        .native_with_mod_block_entity(pos, |be| {
            if be.id != bus::BLOCK || be.nbt.contains_key(&key("cable")) {
                return false;
            }
            be.nbt.insert(key("cable"), Tag::Compound(part));
            true
        })
        .unwrap_or(false);
    if !added {
        cleanup_empty(game, pos, wet);
        return;
    }
    super::ae2_grid_glue::node_placed(game.dim.id(), pos);
    super::ae2_grid_glue::cables_changed(game);
    // PartPlacement checks the full host shape with no excluded entity.
    // Do not use the engine's static collision cache for this multipart host.
    let shape = horizon_client_render::ae2_cable_view::collision(
        &game.world,
        &game.pack.assets.registry,
        pos,
    );
    let unobstructed = shape.is_some_and(|shape| {
        let shape = shape.moved(horizon_core::geometry::Vec3::new(
            f64::from(pos[0]),
            f64::from(pos[1]),
            f64::from(pos[2]),
        ));
        game.with_level(|_, level, _| {
            horizon_world::entity_obstruction::is_unobstructed(
                &mut super::BuildingObstruction(level.building_blocker_boxes()),
                None,
                &shape,
            )
            .unwrap_or(false)
        })
    });
    if !unobstructed {
        game.native_with_mod_block_entity(pos, |be| {
            be.nbt.remove(&key("cable"));
        });
        cleanup_empty(game, pos, wet);
        super::ae2_grid_glue::cables_changed(game);
        return;
    }
    if let Some(log) = &mut game.world.block_log {
        log.push((pos[0], pos[1], pos[2]));
    }
    game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
    game.with_level(|_, level, _| {
        level.native_play_sound(
            "minecraft:block.glass.place",
            pos.map(|v| f64::from(v) + 0.5),
            4,
            1.0,
            0.8,
        );
    });
    game.consume_held();
}
fn cleanup_empty(game: &mut Game<'_>, pos: [i32; 3], wet: bool) {
    // CableBusBlockEntity.cleanup -> fluid legacy block, not the replaced grass.
    let block = if wet {
        "minecraft:water"
    } else {
        "minecraft:air"
    };
    if let Some(state) = game.pack.assets.registry.default_state(block) {
        let dimension = game.dim.id().to_owned();
        game.native_set_block(&dimension, pos, state, 3);
    }
}
pub(super) fn drops(entity: &BlockEntity, block: &str) -> Option<Vec<ItemStack>> {
    if block != bus::BLOCK {
        return None;
    }
    let BlockEntity::Mod(be) = entity else {
        return Some(vec![]);
    };
    if be.id != bus::BLOCK {
        return Some(vec![]);
    }
    let mut result = vec![];
    if let Ok(saved) = bus::Saved::read(&be.nbt)
        && let Some(part) = saved.parts.center()
        && Cable::from_item(&part.id).is_some()
    {
        let mut stack = ItemStack::new(&part.id, 1);
        // IPart.addPartDrop exports dismantle settings, never gn energy/owner
        // and never the original item's arbitrary custom_data/components.
        if let Some(Tag::String(name)) = part.data.get(&key("customName"))
            && let Ok(value) = serde_json::from_str::<serde_json::Value>(&name.to_string_lossy())
        {
            stack.data_mut().custom_name = Some(horizon_client::scoreboard::json_to_tag(&value));
        }
        result.push(stack);
    }
    Some(result)
}
pub(super) fn remembered(pending: &super::spill::Pending, pos: [i32; 3]) -> Vec<ItemStack> {
    let Some((id, fields)) = pending.copied.get(&pos) else {
        return vec![];
    };
    if id != bus::BLOCK {
        return vec![];
    }
    let Some(Tag::List(items)) = fields.get(&key("ae2:cable_part_drops")) else {
        return vec![];
    };
    items.iter().filter_map(save::item_from_nbt).collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    use horizon_client::containers::ModBlockEntity;
    #[test]
    fn world_owner_mapping_uses_original_arrays_and_survives_a_new_reader() {
        let parent = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let folder = parent.join(format!(
            "ae2-player-map-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&folder).unwrap();
        let path = folder.join("ae2_players.dat");
        assert_eq!(owner_at(&path, u128::MAX).unwrap(), 0);
        assert_eq!(owner_at(&path, 0).unwrap(), 1);
        assert_eq!(owner_at(&path, u128::MAX).unwrap(), 0);
        let saved = save::read_dat(&path).unwrap();
        let data = saved.get("data").unwrap();
        assert_eq!(data.get("playerIds"), Some(&Tag::IntArray(vec![0, 1])));
        assert_eq!(
            data.get("profileIds"),
            Some(&Tag::LongArray(vec![-1, -1, 0, 0]))
        );
        let corrupt = Tag::compound([(
            "data",
            Tag::compound([
                ("playerIds", Tag::IntArray(vec![0])),
                ("profileIds", Tag::LongArray(vec![])),
            ]),
        )]);
        save::write_dat(&path, corrupt).unwrap();
        let original = std::fs::read(&path).unwrap();
        assert!(owner_at(&path, 5).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn cable_drops_export_name_but_not_node_energy_or_owner_and_remain_position_scoped() {
        let mut be = ModBlockEntity {
            id: bus::BLOCK.into(),
            ..Default::default()
        };
        be.nbt.insert(
            key("cable"),
            Tag::compound([
                ("id", Tag::String("ae2:red_smart_cable".into())),
                (
                    "customName",
                    Tag::String(r#"{"text":"Red cable","color":"red"}"#.into()),
                ),
                (
                    "gn",
                    Tag::compound([("p", Tag::Int(8)), ("e", Tag::Double(25.0f64.to_bits()))]),
                ),
            ]),
        );
        let entity = BlockEntity::Mod(Box::new(be));
        let before = entity.clone();
        let dropped = drops(&entity, bus::BLOCK).unwrap();
        assert_eq!(dropped.len(), 1);
        assert_eq!(&*dropped[0].item, "ae2:red_smart_cable");
        assert_eq!(dropped[0].count, 1);
        let components = horizon_client::item_data::encode_components(&dropped[0]).unwrap();
        assert_eq!(
            components.get("minecraft:custom_name").unwrap().get("text"),
            Some(&Tag::String("Red cable".into()))
        );
        assert!(components.get("gn").is_none());
        assert!(components.get("ae2:stored_energy").is_none());
        assert_eq!(
            save::encode_block_entity((1, 2, 3), &entity, bus::BLOCK),
            save::encode_block_entity((1, 2, 3), &before, bus::BLOCK)
        );
        let mut pending = super::super::spill::Pending::default();
        pending.record([1, 2, 3], &entity, bus::BLOCK, None);
        assert_eq!(remembered(&pending, [1, 2, 3]).len(), 1);
        assert!(remembered(&pending, [2, 2, 3]).is_empty());
        pending.clear([1, 2, 3]);
        assert!(remembered(&pending, [1, 2, 3]).is_empty());
    }
}
