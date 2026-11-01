// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_item_decorations(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    stack: &horizon_client::items::ItemStack,
    x: f32,
    y: f32,
    count_text: &Option<String>,
) -> core::ops::ControlFlow<()> {
    if crate::game::ae2_item_energy_view::decorations(game, batch, stack, x, y, count_text) {
        return core::ops::ControlFlow::Break(());
    }
    core::ops::ControlFlow::Continue(())
}
