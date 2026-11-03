// SPDX-License-Identifier: LGPL-3.0-or-later
// Client PartPlacement for supported centers; no server grid or owner-map writes.
use super::Game;
use crate::game::{
    ae2_cable::Cable,
    ae2_cable_items,
    ae2_part_placement::Placement,
    ae2_part_use::{self, Context, UseResult},
};
use horizon_client::{containers::BlockEntity, items::ItemStack};
use horizon_core::{geometry::Vec3, position::BlockPos};
use horizon_protocol::connection::Counters;
use horizon_world::{
    fluid::{FluidKind, FluidState},
    nbt::{Compound, Tag},
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
struct Pending {
    sequence: i32,
    known: Option<BlockEntity>,
}
#[derive(Default)]
struct Hosts(BTreeMap<[i32; 3], Pending>);
impl Hosts {
    fn retain(&mut self, pos: [i32; 3], sequence: i32, before: Option<BlockEntity>) {
        self.0
            .entry(pos)
            .and_modify(|p| p.sequence = sequence)
            .or_insert(Pending {
                sequence,
                known: before,
            });
    }
    fn verified(&mut self, pos: [i32; 3], entity: BlockEntity) {
        if let Some(p) = self.0.get_mut(&pos) {
            p.known = Some(entity);
        }
    }
    fn defer(&mut self, pos: [i32; 3], sequence: i32) {
        if let Some(p) = self.0.get_mut(&pos) {
            p.sequence = p.sequence.max(sequence);
        }
    }
    fn settle(&mut self, sequence: i32) -> Vec<([i32; 3], Option<BlockEntity>)> {
        let keys: Vec<_> = self
            .0
            .iter()
            .filter(|(_, p)| p.sequence <= sequence)
            .map(|(p, _)| *p)
            .collect();
        keys.into_iter()
            .map(|p| (p, self.0.remove(&p).unwrap().known))
            .collect()
    }
}
struct Preview {
    connection: Arc<Mutex<Option<Arc<Counters>>>>,
    hosts: Hosts,
}
thread_local! { static PREVIEW: RefCell<Option<Preview>> = const { RefCell::new(None) }; }
fn with_hosts<T>(game: &Game<'_>, f: impl FnOnce(&mut Hosts) -> T) -> Option<T> {
    let connection = game.remote.as_ref()?.session.counters.clone();
    PREVIEW.with(|state| {
        let mut state = state.borrow_mut();
        if state
            .as_ref()
            .is_none_or(|s| !Arc::ptr_eq(&s.connection, &connection))
        {
            *state = Some(Preview {
                connection,
                hosts: Hosts::default(),
            });
        }
        Some(f(&mut state.as_mut().unwrap().hosts))
    })
}
pub(super) fn reset() {
    PREVIEW.with(|s| *s.borrow_mut() = None);
}
pub(super) fn verified(game: &Game<'_>, pos: [i32; 3], entity: &BlockEntity) {
    with_hosts(game, |hosts| hosts.verified(pos, entity.clone()));
}
pub(super) fn retain(game: &Game<'_>, pos: [i32; 3], sequence: i32) {
    with_hosts(game, |hosts| {
        hosts.retain(
            pos,
            sequence,
            game.world
                .block_entities
                .get(&(pos[0], pos[1], pos[2]))
                .cloned(),
        );
    });
}
pub(super) fn finish(game: &mut Game<'_>, sequence: i32) {
    let pending = with_hosts(game, |hosts| {
        // A later native action (for example a predicted break) can coalesce
        // the same block. Its BE must wait for that action too, not the older
        // part placement's acknowledgement.
        if let Some(remote) = &game.remote {
            for (&pos, &(later, _)) in &remote.predictions {
                hosts.defer(pos, later);
            }
        }
        hosts.settle(sequence)
    })
    .unwrap_or_default();
    for (pos, known) in pending {
        if game.native_block(pos) != "ae2:cable_bus" {
            continue;
        }
        let key = (pos[0], pos[1], pos[2]);
        if let Some(known) =
            known.filter(|e| matches!(e,BlockEntity::Mod(be) if be.id == "ae2:cable_bus"))
        {
            game.world.block_entities.insert(key, known);
        } else {
            game.world.block_entities.remove(&key);
            game.sync_block_entity(key, game.world.block(pos[0], pos[1], pos[2]));
        }
    }
}

pub(super) fn use_on(
    game: &mut Game<'_>,
    stack: &ItemStack,
    hit: &crate::game::Hit,
) -> Option<UseResult> {
    if Cable::from_item(&stack.item).is_none() || stack.count == 0 {
        return None;
    }
    let info = game.items().get(&stack.item)?;
    if game
        .cooldowns
        .is_on_cooldown(&horizon_client::cooldowns::group(
            stack,
            info.use_cooldown.as_ref(),
        ))
    {
        return None;
    }
    if !crate::game::ae2_adventure_native::may_use(game, stack, hit.pos) {
        return None;
    }
    let seq = game.remote.as_ref()?.sequence;
    struct Client<'a, 'g> {
        game: &'a mut Game<'g>,
        stack: &'a ItemStack,
        hit: &'a crate::game::Hit,
        seq: i32,
    }
    impl Context for Client<'_, '_> {
        type Placement = Placement;
        fn is_part(&self) -> bool {
            true
        }
        fn client_side(&self) -> bool {
            true
        }
        fn placement(&mut self) -> Option<Placement> {
            let key = self.game.native_mod_key(
                "ae2",
                crate::game::ae2_ctrl::BINDING,
                crate::game::ae2_ctrl::DEFAULT_KEY,
            );
            let ctrl = self
                .game
                .physical_keys
                .iter()
                .any(|k| horizon_client::keybinds::key_name(*k) == Some(key.as_str()));
            ae2_cable_items::target(self.game, self.hit.pos, self.hit.face, ctrl)
        }
        fn place(&mut self, target: Placement) -> bool {
            place(self.game, self.stack, target.pos, self.seq)
        }
        fn immediate_update(&mut self) {} // Platform only sends from a ServerPlayer.
        fn consume(&mut self) {
            unreachable!("a client does not shrink a part item")
        }
    }
    Some(ae2_part_use::use_item(&mut Client {
        game,
        stack,
        hit,
        seq,
    }))
}
fn place(game: &mut Game<'_>, stack: &ItemStack, pos: [i32; 3], seq: i32) -> bool {
    const HOST: &str = "ae2:cable_bus";
    if pos[1] < game.world.shape.min_y || pos[1] >= game.world.shape.max_y() {
        return false;
    }
    let before = game.world.block(pos[0], pos[1], pos[2]);
    let was_host = game.native_block(pos) == HOST;
    let wet = game
        .pack
        .assets
        .registry
        .state(before)
        .and_then(|s| FluidState::from_block(s).ok())
        .is_some_and(|f| f.kind() == FluidKind::Water);
    let state = if was_host {
        before
    } else {
        let Some(mut state) = game.pack.assets.registry.default_state(HOST) else {
            return false;
        };
        if wet {
            state = game
                .with_props(state, &[("waterlogged", "true")])
                .unwrap_or(state);
        }
        state
    };
    // Register the real known server block before mutating the preview. The
    // normal sequence acknowledgement removes a refused host, including CTRL's
    // reverse target which vanilla's two block resends do not cover.
    retain(game, pos, seq);
    game.predict_blocks(&vec![(BlockPos::new(pos[0], pos[1], pos[2]), state)], seq);
    if game.world.block(pos[0], pos[1], pos[2]) != state {
        return false;
    }
    let at = (pos[0], pos[1], pos[2]);
    if !was_host {
        let Some(be) = game.new_block_entity(HOST) else {
            return false;
        };
        game.world.block_entities.insert(at, be);
    }
    let Some(BlockEntity::Mod(be)) = game.world.block_entities.get_mut(&at) else {
        return false;
    };
    if be.id != HOST || be.nbt.contains_key(&"cable".into()) {
        return false;
    }
    let mut part = Compound::new();
    part.insert("id".into(), Tag::String(stack.item.as_ref().into()));
    if let Some(name) = stack.data().and_then(|d| d.custom_name.as_ref()) {
        let json = horizon_client::scoreboard::tag_to_json(name).to_string();
        part.insert("customName".into(), Tag::String(json.as_str().into()));
    }
    be.nbt.insert("cable".into(), Tag::Compound(part));
    // A freshly constructed client CablePart has no streamed connections yet.
    be.data.clear();
    let shape = horizon_client_render::ae2_cable_view::collision(
        &game.world,
        &game.pack.assets.registry,
        pos,
    );
    let clear = shape.is_some_and(|shape| {
        let shape = shape.moved(Vec3::new(
            f64::from(pos[0]),
            f64::from(pos[1]),
            f64::from(pos[2]),
        ));
        game.with_level(|_, level, _| {
            horizon_world::entity_obstruction::is_unobstructed(
                &mut crate::game::BuildingObstruction(level.building_blocker_boxes()),
                None,
                &shape,
            )
            .unwrap_or(false)
        })
    });
    if !clear {
        let name = if wet {
            "minecraft:water"
        } else {
            "minecraft:air"
        };
        if let Some(state) = game.pack.assets.registry.default_state(name) {
            game.predict_blocks(&vec![(BlockPos::new(pos[0], pos[1], pos[2]), state)], seq);
        }
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entity(value: i32) -> BlockEntity {
        BlockEntity::Mod(Box::new(horizon_client::containers::ModBlockEntity {
            id: "ae2:cable_bus".into(),
            nbt: [("marker".into(), Tag::Int(value))].into(),
            ..Default::default()
        }))
    }
    fn marker(entity: &Option<BlockEntity>) -> Option<i32> {
        let Some(BlockEntity::Mod(be)) = entity else {
            return None;
        };
        let Some(Tag::Int(n)) = be.nbt.get(&"marker".into()) else {
            return None;
        };
        Some(*n)
    }
    #[test]
    fn ae2_predicted_hosts_keep_the_latest_sequence_and_the_original_server_data() {
        let mut hosts = Hosts::default();
        let p = [1, 2, 3];
        hosts.retain(p, 1, Some(entity(7)));
        hosts.retain(p, 2, Some(entity(999)));
        assert!(hosts.settle(1).is_empty());
        let settled = hosts.settle(2);
        assert_eq!(settled.len(), 1);
        assert_eq!(marker(&settled[0].1), Some(7));
        assert!(hosts.settle(100).is_empty());
    }
    #[test]
    fn ae2_verified_host_replaces_preview_without_corrupting_other_positions() {
        let mut hosts = Hosts::default();
        let p = [1, 2, 3];
        let q = [4, 5, 6];
        hosts.retain(p, 3, None);
        hosts.retain(q, 4, Some(entity(7)));
        hosts.verified(p, entity(9));
        hosts.verified([0; 3], entity(100));
        let settled = hosts.settle(3);
        assert_eq!(settled.len(), 1);
        assert_eq!(marker(&settled[0].1), Some(9));
        assert_eq!(marker(&hosts.settle(4)[0].1), Some(7));
    }
    #[test]
    fn ae2_later_native_action_keeps_host_until_its_ack_and_accepts_hidden_server_data() {
        let mut hosts = Hosts::default();
        let p = [1, 2, 3];
        hosts.retain(p, 1, None);
        hosts.defer(p, 2);
        assert!(hosts.settle(1).is_empty());
        // A predicted break hides the block, but its server BE packet is
        // still authoritative and must survive a subsequently refused break.
        hosts.verified(p, entity(9));
        hosts.defer(p, 1);
        assert!(hosts.settle(1).is_empty());
        assert_eq!(marker(&hosts.settle(2)[0].1), Some(9));
        // A standalone break of an already confirmed host needs the same
        // original-data retention even when no placement preview preceded it.
        hosts.retain(p, 3, Some(entity(11)));
        assert_eq!(marker(&hosts.settle(3)[0].1), Some(11));
    }
}
