fn ae2_flush_before_destroy(game: &mut crate::game::Game<'_>, pos: [i32; 3]) {
    if game.native_block(pos) == "ae2:drive" {
        crate::game::ae2_grid_glue::flush_drives(game);
    }
}
