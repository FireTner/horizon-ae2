// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_level_pre(game: &mut crate::game::Game<'_>, dimension: &str) {
    crate::game::ae2_grid_glue::pre_tick(game, dimension);
}
