fn ae2_automation_changed(game: &mut crate::game::Game<'_>, pos: (i32, i32, i32)) {
    if matches!(game.world.block_entities.get(&pos), Some(horizon_client::containers::BlockEntity::Mod(be)) if be.id == "ae2:drive")
    {
        crate::game::ae2_grid_glue::flush_drives(game);
    }
}
