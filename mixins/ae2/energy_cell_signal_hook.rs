fn ae2_energy_cell_signal(
    level: &crate::game::GameLevel<'_>,
    pos: horizon_core::position::BlockPos,
) -> core::ops::ControlFlow<i32> {
    if let Some(horizon_client::containers::BlockEntity::Mod(be)) =
        level.world.block_entities.get(&(pos.x, pos.y, pos.z))
        && let Some(signal) = ae2_energy_cell_glue::signal(&be.id, &be.nbt)
    {
        return core::ops::ControlFlow::Break(signal);
    }
    core::ops::ControlFlow::Continue(())
}
