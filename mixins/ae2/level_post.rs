// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_level_post(game: &mut crate::game::Game<'_>, dimension: &str) {
    crate::game::ae2_grid_glue::post_tick(game, dimension);
}
