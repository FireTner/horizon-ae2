// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::ae2_glue::tick(game, pos, entity);
}
