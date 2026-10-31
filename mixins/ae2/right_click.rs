// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_right_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    crate::game::ae2_glue::right_click(game, pos, canceled);
}
