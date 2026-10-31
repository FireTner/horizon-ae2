// SPDX-License-Identifier: LGPL-3.0-or-later
// Drive screen geometry/background from the pinned AE2 ScreenStyle resources.
// Priority sub-screen, exact widget rendering and broader cell backends remain
// separate tasks. The server owns item movement and filtered cell slots.
use super::{Game, Screen, native_hooks::NativeScreen};
use horizon_client::{
    hud::HudBatch,
    items::ItemStack,
    lan::menu::{ae2_drive_context, ae2_slot_rules as rules},
};
use std::cell::Cell;
thread_local! { static OPEN_DRIVE: Cell<Option<(i32,i32,i32)>> = const { Cell::new(None) }; }
pub(super) fn screen(block: &str, out: &mut Option<NativeScreen>) {
    if block == "ae2:drive" {
        *out = Some(NativeScreen {
            width: rules::WIDTH,
            height: rules::HEIGHT,
            slots: (0..10).map(rules::drive_position).collect(),
            inventory_y: rules::INVENTORY_Y,
            inventory_x: 8.0,
            own_labels: false,
        });
    }
}
pub(super) fn draw_full(game: &Game<'_>, batch: &mut HudBatch, origin: [f32; 2]) -> bool {
    if game.screen != Screen::Inventory
        || !OPEN_DRIVE.get().is_some_and(|p| game.container == Some(p))
    {
        return false;
    }
    if !game.native_sprite(
        batch,
        "ae2:gui/drive",
        [0.0, 0.0, rules::WIDTH, rules::HEIGHT],
        [origin[0], origin[1], rules::WIDTH, rules::HEIGHT],
    ) {
        return false;
    }
    game.native_text(
        batch,
        &game.translate("gui.ae2.Drive", "ME Drive"),
        [origin[0] + 8.0, origin[1] + 6.0],
        0x404040,
        false,
    );
    game.native_text(
        batch,
        &game.translate("container.inventory", "Inventory"),
        [origin[0] + 8.0, origin[1] + 106.0],
        0x404040,
        false,
    );
    true
}
pub(super) fn opened(game: &Game<'_>, key: (i32, i32, i32), target: Option<(i32, i32, i32)>) {
    let drive = target.is_some_and(|p| game.native_block([p.0, p.1, p.2]) == "ae2:drive");
    OPEN_DRIVE.set(drive.then_some(key));
    ae2_drive_context::set_client_drive(drive);
    ae2_drive_context::set_remote_window(game.remote.is_some());
}
pub(super) fn closed() {
    OPEN_DRIVE.set(None);
    ae2_drive_context::set_client_drive(false);
    ae2_drive_context::set_remote_window(false);
}
pub(super) fn slot_rule(game: &Game<'_>, slot: usize, stack: &ItemStack) -> Option<(bool, u32)> {
    let drive = game.screen == Screen::Inventory
        && OPEN_DRIVE.get().is_some_and(|p| game.container == Some(p));
    if drive && (300..318).contains(&slot) {
        return Some(if slot < 310 {
            rules::drive_slot(&stack.item, stack.count)
        } else {
            (false, 0)
        });
    }
    None
}
