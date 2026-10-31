// SPDX-License-Identifier: LGPL-3.0-or-later
// Native full-block network lifecycle. Only server level hooks modify networks.
// Every dimension owns independent topology and EnergyService state.

use super::{
    Game,
    ae2_grid::Direction,
    ae2_network::{Kind, Network, Snapshot},
};
use horizon_client::containers::BlockEntity;
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::collections::BTreeMap;
use std::sync::Mutex;

static NETWORKS: Mutex<BTreeMap<String, Network>> = Mutex::new(BTreeMap::new());

pub(super) fn owns(block: &str) -> bool {
    Kind::from_block(block).is_some()
}
fn get_double(nbt: &Compound, key: &str) -> f64 {
    match nbt.get(&JavaString::from(key)) {
        Some(Tag::Double(v)) if f64::from_bits(*v).is_finite() => f64::from_bits(*v).max(0.0),
        _ => 0.0,
    }
}
fn direction(game: &Game<'_>, pos: [i32; 3]) -> Direction {
    match game.native_block_property(pos, "facing").as_deref() {
        Some("down") => Direction::Down,
        Some("up") => Direction::Up,
        Some("south") => Direction::South,
        Some("west") => Direction::West,
        Some("east") => Direction::East,
        _ => Direction::North,
    }
}
fn snapshots(game: &Game<'_>) -> Vec<Snapshot> {
    let mut snapshots = vec![];
    for (&(x, y, z), entity) in &game.world.block_entities {
        let BlockEntity::Mod(be) = entity else {
            continue;
        };
        let Some(kind) = Kind::from_block(&be.id) else {
            continue;
        };
        if !game.world.loaded(x, z) || game.native_block([x, y, z]) != kind.block() {
            continue;
        }
        let saved_buffer = match be.nbt.get(&JavaString::from("proxy")) {
            Some(Tag::Compound(node)) => get_double(node, "e"),
            _ => 0.0,
        };
        snapshots.push(Snapshot {
            pos: [x, y, z],
            kind,
            front: direction(game, [x, y, z]),
            stored: get_double(&be.nbt, "internalCurrentPower"),
            saved_buffer,
        });
    }
    snapshots
}
pub(super) fn pre_tick(game: &mut Game<'_>, dimension: &str) {
    let snapshots = snapshots(game);
    let mut networks = NETWORKS.lock().unwrap_or_else(|e| e.into_inner());
    let net = networks.entry(dimension.to_owned()).or_default();
    net.sync(snapshots);
    net.begin_tick();
}
pub(super) fn post_tick(game: &mut Game<'_>, dimension: &str) {
    // Block placement, removal and /data may run inside the level tick. Notice
    // those changes before persisting; unchanged NBT is not an external refill
    // and must never restore energy already extracted during this tick.
    let snapshots = snapshots(game);
    let states = {
        let mut networks = NETWORKS.lock().unwrap_or_else(|e| e.into_inner());
        let Some(net) = networks.get_mut(dimension) else {
            return;
        };
        net.sync(snapshots);
        net.end_tick()
    };
    for state in states {
        let [x, y, z] = state.pos;
        let Some(BlockEntity::Mod(be)) = game.world.block_entities.get_mut(&(x, y, z)) else {
            continue;
        };
        if be.id != state.kind.block() {
            continue;
        }
        let before = be.nbt.clone();
        if let Some(energy) = state.stored
            && state.kind != Kind::CreativeEnergyCell
        {
            be.nbt.insert(
                JavaString::from("internalCurrentPower"),
                Tag::Double(energy.to_bits()),
            );
        }
        let proxy = be
            .nbt
            .entry(JavaString::from("proxy"))
            .or_insert_with(|| Tag::Compound(Compound::new()));
        if let Tag::Compound(proxy) = proxy {
            proxy.entry(JavaString::from("p")).or_insert(Tag::Int(-1));
            if state.buffer_share > 0.0 {
                proxy.insert(
                    JavaString::from("e"),
                    Tag::Double(state.buffer_share.to_bits()),
                );
            } else {
                proxy.remove(&JavaString::from("e"));
            }
        }
        if be.nbt != before {
            game.world.modified.insert((x >> 4, z >> 4));
        }
    }
}
pub(super) fn extract(
    dimension: &str,
    pos: [i32; 3],
    amount: f64,
    multiplier: f64,
    commit: bool,
) -> f64 {
    let mut networks = NETWORKS.lock().unwrap_or_else(|e| e.into_inner());
    networks
        .get_mut(dimension)
        .map_or(0.0, |n| n.extract(pos, amount, multiplier, commit))
}
pub(super) fn reset() {
    NETWORKS.lock().unwrap_or_else(|e| e.into_inner()).clear();
}
