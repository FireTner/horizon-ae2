fn ae2_automation_flush(
    game: &mut crate::game::Game<'_>,
    view: &crate::game::machines::View,
    slot: usize,
) {
    if let Some((pos, _)) = view.locate(slot)
        && matches!(game.world.block_entities.get(&pos), Some(horizon_client::containers::BlockEntity::Mod(be)) if be.id == "ae2:drive")
    {
        crate::game::ae2_grid_glue::flush_drives(game);
    }
}
