// SPDX-License-Identifier: LGPL-3.0-or-later
// Scope the native server's existing complete click engine to a Drive's rules.
// Menu metadata is refreshed from the authoritative block on every packet.
// No policy is inferred from a chest size, hidden-slot pattern or title.

use crate::{
    containers::BlockEntity,
    lan::{hub::Hub, menu::Slots},
    world::World,
};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
thread_local! {
    static MENUS: RefCell<BTreeMap<u128,(i32,bool)>> = const { RefCell::new(BTreeMap::new()) };
    static CLIENT_DRIVE: Cell<bool> = const { Cell::new(false) };
    static REMOTE_WINDOW: Cell<bool> = const { Cell::new(false) };
}
pub fn client_drive() -> bool {
    CLIENT_DRIVE.get()
}
pub fn set_client_drive(drive: bool) {
    CLIENT_DRIVE.set(drive);
}
pub fn remote_window() -> bool {
    REMOTE_WINDOW.get()
}
pub fn set_remote_window(remote: bool) {
    REMOTE_WINDOW.set(remote);
}
pub fn begin(hub: &Hub, conn: u32, window: i32, world: &World) {
    let guest = hub.guest(conn);
    let drive = guest
        .and_then(|g| g.menu.as_ref())
        .filter(|m| window != 0 && m.window == window)
        .and_then(|m| match m.slots {
            Slots::Block(pos) => world.block_entities.get(&pos),
            _ => None,
        })
        .is_some_and(|be| matches!(be,BlockEntity::Mod(be) if be.id == "ae2:drive"));
    if let Some(guest) = guest {
        MENUS.with_borrow_mut(|menus| menus.insert(guest.uuid, (window, drive)));
    }
}
pub fn active(actor: u128, window: Option<i32>) -> bool {
    MENUS.with_borrow(|menus| {
        menus
            .get(&actor)
            .is_some_and(|(id, drive)| *drive && window == Some(*id))
    })
}
pub fn reset() {
    MENUS.with_borrow_mut(BTreeMap::clear);
    set_client_drive(false);
    set_remote_window(false);
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        items::{ItemStack, Items},
        lan::{
            Joined, PlayerSender,
            guest::Guest,
            menu::{self, Menu, Rules, View, input},
        },
    };
    use horizon_protocol::packets::configuration::ClientInformation;
    use horizon_world::loot::PackOverlay;
    use std::sync::{Arc, atomic::AtomicI32, mpsc};
    struct ItemRules(Items);
    impl Rules for ItemRules {
        fn items(&self) -> &Items {
            &self.0
        }
        fn max_stack(&self, id: &str) -> u32 {
            self.0.get(id).map_or(64, |i| i.max_stack)
        }
        fn armor_index(&self, _: &str) -> Option<usize> {
            None
        }
    }
    fn fixture() -> (Guest, ItemRules) {
        let root = std::path::PathBuf::from(
            std::env::var_os("HORIZON_RESOURCES")
                .expect("resource-backed AE2 menu tests require HORIZON_RESOURCES"),
        );
        let rules = ItemRules(Items::catalog(&root, &PackOverlay::NONE).unwrap());
        let (tx, _rx) = mpsc::channel();
        let mut guest = Guest::new(
            Joined {
                conn: 2,
                uuid: 2,
                name: "Alex".into(),
                properties: vec![],
                info: ClientInformation::new(8),
                brand: "test".into(),
                sender: PlayerSender::new(tx, Arc::default()),
                latency: Arc::new(AtomicI32::new(0)),
                address: "127.0.0.1:25565".parse().unwrap(),
            },
            2,
            [0.0, 64.0, 0.0],
            8,
        );
        guest.menu = Some(Menu {
            window: 3,
            kind: 0,
            pos: Some((0, 64, 0)),
            slots: Slots::Block((0, 64, 0)),
            size: 18,
            state: 0,
            sent: vec![],
            drag: None,
            sent_data: vec![],
            hidden: (10..18).collect(),
        });
        MENUS.with_borrow_mut(|m| m.insert(2, (3, true)));
        (guest, rules)
    }
    #[test]
    fn menu_metadata_is_actor_and_window_specific_and_replaced_on_new_packet() {
        MENUS.with_borrow_mut(|m| m.insert(1, (3, true)));
        assert!(active(1, Some(3)));
        assert!(!active(2, Some(3)));
        assert!(!active(1, Some(4)));
        MENUS.with_borrow_mut(|m| m.insert(2, (3, false)));
        assert!(active(1, Some(3)));
        MENUS.with_borrow_mut(|m| m.insert(1, (4, false)));
        assert!(!active(1, Some(3)));
        assert!(!active(1, Some(4)));
        reset();
        assert!(MENUS.with_borrow(BTreeMap::is_empty));
    }
    #[test]
    fn server_pickup_rejects_items_limits_cells_and_quick_move_prefers_main_inventory() {
        let (mut guest, rules) = fixture();
        let mut container = vec![None; 18];
        let mut drag = None;
        let mut view = View {
            guest: &mut guest,
            container: Some(&mut container),
            shulker: false,
            hidden: &[10, 11, 12, 13, 14, 15, 16, 17],
        };
        view.guest.carried = Some(ItemStack::new("minecraft:stone", 5));
        menu::click(&mut view, &mut drag, 0, 0, input::PICKUP, &rules);
        assert!(view.get(0).is_none());
        assert_eq!(view.guest.carried.as_ref().unwrap().count, 5);
        view.guest.carried = Some(ItemStack::new("ae2:item_storage_cell_1k", 3));
        menu::click(&mut view, &mut drag, 0, 0, input::PICKUP, &rules);
        assert_eq!(view.get(0).unwrap().count, 1);
        assert_eq!(view.guest.carried.as_ref().unwrap().count, 2);
        view.guest.carried = None;
        menu::click(&mut view, &mut drag, 0, 0, input::QUICK_MOVE, &rules);
        assert!(view.get(0).is_none());
        assert_eq!(
            &*view.guest.inventory[9].as_ref().unwrap().item,
            "ae2:item_storage_cell_1k"
        );
        assert!(view.guest.inventory[8].is_none());
        menu::click(&mut view, &mut drag, 18, 0, input::QUICK_MOVE, &rules);
        assert_eq!(view.get(0).unwrap().count, 1);
        assert!(view.guest.inventory[9].is_none());
        reset();
    }
    #[test]
    fn server_drag_and_hotbar_swaps_keep_cell_limits_and_hidden_padding_empty() {
        let (mut guest, rules) = fixture();
        let mut container = vec![None; 18];
        let mut drag = None;
        let mut view = View {
            guest: &mut guest,
            container: Some(&mut container),
            shulker: false,
            hidden: &[10, 11, 12, 13, 14, 15, 16, 17],
        };
        view.guest.carried = Some(ItemStack::new("ae2:item_storage_cell_1k", 3));
        menu::click(&mut view, &mut drag, -999, 0, input::QUICK_CRAFT, &rules);
        for slot in [0, 1, 9, 10] {
            menu::click(&mut view, &mut drag, slot, 1, input::QUICK_CRAFT, &rules);
        }
        menu::click(&mut view, &mut drag, -999, 2, input::QUICK_CRAFT, &rules);
        assert!(view.guest.carried.is_none());
        for slot in [0, 1, 9] {
            assert_eq!(view.get(slot).unwrap().count, 1);
        }
        assert!(view.get(10).is_none());
        view.guest.inventory[0] = Some(ItemStack::new("minecraft:stone", 1));
        menu::click(&mut view, &mut drag, 0, 0, input::SWAP, &rules);
        assert_eq!(&*view.get(0).unwrap().item, "ae2:item_storage_cell_1k");
        view.guest.inventory[0] = Some(ItemStack::new("ae2:fluid_storage_cell_4k", 1));
        menu::click(&mut view, &mut drag, 0, 0, input::SWAP, &rules);
        assert_eq!(&*view.get(0).unwrap().item, "ae2:fluid_storage_cell_4k");
        assert_eq!(
            &*view.guest.inventory[0].as_ref().unwrap().item,
            "ae2:item_storage_cell_1k"
        );
        reset();
    }
}
