// SPDX-License-Identifier: LGPL-3.0-or-later
// The source CTRL attachment, carried through Horizon's authenticated mod channel.
use super::{
    Game,
    ae2_ctrl::{self as ctrl, Controls},
};
use horizon_protocol::connection::Counters;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
type Connection = Arc<Mutex<Option<Arc<Counters>>>>;
struct Client {
    connection: Connection,
    sent: Option<bool>,
}
thread_local! {static CLIENT:RefCell<Option<Client>>=const {RefCell::new(None)};}
static PLAYERS: Mutex<BTreeMap<PathBuf, Controls>> = Mutex::new(BTreeMap::new());
pub(super) fn receive(world: &Path, sender: u128, data: &[u8]) {
    PLAYERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .entry(world.to_owned())
        .or_default()
        .receive(sender, data);
}
pub(super) fn reset_world(world: &Path) {
    PLAYERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(world);
}
pub(super) fn reset_player(world: &Path, player: u128) {
    if let Some(keys) = PLAYERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(world)
    {
        keys.reset(player);
    }
}
pub(super) fn holding(game: &Game<'_>) -> bool {
    let Some(player) = game.native_acting_player() else {
        return false;
    };
    PLAYERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&game.world_root)
        .is_some_and(|keys| keys.down(player.uuid))
}
fn chosen(game: &Game<'_>) -> String {
    game.native_mod_key("ae2", ctrl::BINDING, ctrl::DEFAULT_KEY)
}
fn send(game: &mut Game<'_>, down: bool) {
    let Some(remote) = game
        .remote
        .as_ref()
        .filter(|r| r.joined && r.mod_net && game.player.is_some())
    else {
        CLIENT.with(|c| *c.borrow_mut() = None);
        return;
    };
    let connection = remote.session.counters.clone();
    let changed = CLIENT.with(|c| {
        let mut state = c.borrow_mut();
        if state
            .as_ref()
            .is_none_or(|old| !Arc::ptr_eq(&old.connection, &connection))
        {
            *state = Some(Client {
                connection,
                sent: None,
            });
        }
        state.as_ref().unwrap().sent != Some(down)
    });
    if changed
        && game
            .native_net_send_to_server(ctrl::CHANNEL, &ctrl::encode(down))
            .is_ok()
    {
        CLIENT.with(|c| {
            if let Some(state) = c.borrow_mut().as_mut() {
                state.sent = Some(down);
            }
        });
    }
}
pub(super) fn key(game: &mut Game<'_>, name: &str, pressed: bool) {
    if name == chosen(game) {
        send(game, pressed);
    }
}
pub(super) fn frame(game: &mut Game<'_>) {
    // Also initialize each new connection and notice releases behind screens.
    // A strong marker prevents allocator address reuse from hiding a reconnect.
    let key = chosen(game);
    let down = game
        .physical_keys
        .iter()
        .any(|code| horizon_client::keybinds::key_name(*code) == Some(key.as_str()));
    send(game, down);
}
pub(super) fn pick_components(
    pos: [i32; 3],
    entity: &horizon_client::containers::BlockEntity,
) -> horizon_world::nbt::Compound {
    // saved_block_entity canonicalizes only vanilla types and rejects a mod BE.
    // Encode the actual supported host, preserving its authoritative source NBT.
    let saved = horizon_client::save::encode_block_entity(
        (pos[0], pos[1], pos[2]),
        entity,
        "ae2:cable_bus",
    );
    horizon_client::block_entity_item::pick(&saved)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn world_scope_sender_isolation_empty_payload_and_lifecycle_reset() {
        let one = Path::new("ae2-input-test/world-a");
        let two = Path::new("ae2-input-test/world-b");
        reset_world(one);
        reset_world(two);
        receive(one, 1, &[1]);
        receive(two, 1, &[1]);
        receive(one, 2, &[]);
        {
            let state = PLAYERS.lock().unwrap();
            assert!(state[one].down(1));
            assert!(!state[one].down(2));
            assert!(state[two].down(1));
        }
        reset_player(one, 1);
        {
            let state = PLAYERS.lock().unwrap();
            assert!(!state[one].down(1));
            assert!(state[two].down(1));
        }
        reset_world(one);
        {
            let state = PLAYERS.lock().unwrap();
            assert!(!state.contains_key(one));
            assert!(state.contains_key(two));
        }
        reset_world(two);
    }
    #[test]
    fn ae2_cable_pick_preserves_host_data_without_vanilla_canonicalization() {
        let saved = horizon_world::nbt::parse_snbt(
            r#"{id:"ae2:cable_bus",x:6,y:74,z:8,hasRedstone:2,cable:{id:"ae2:cyan_smart_cable",customName:'{"text":"Named center"}',gn:{p:1,e:12.0d}}}"#,
        )
        .unwrap()
        .0;
        let (_, entity) = horizon_client::save::decode_block_entity(&saved).unwrap();
        assert!(horizon_client::block_entity_nbt::canonical(&saved).is_none());
        let patch = pick_components([6, 74, 8], &entity);
        let data = &patch[&horizon_world::nbt::JavaString::from("minecraft:block_entity_data")];
        assert_eq!(data.get("id"), saved.get("id"));
        assert_eq!(data.get("cable"), saved.get("cable"));
        assert_eq!(data.get("hasRedstone"), saved.get("hasRedstone"));
        assert!(data.get("x").is_none());
        assert!(data.get("y").is_none());
        assert!(data.get("z").is_none());
        assert!(!patch.contains_key(&horizon_world::nbt::JavaString::from(
            "minecraft:custom_name"
        )));
    }
}
