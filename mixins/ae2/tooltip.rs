// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_tooltip(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    crate::game::ae2_cells::tooltip(game, stack, lines);
}
