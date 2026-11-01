fn ae2_energy_cell_drop_level(
    level: &mut crate::game::GameLevel<'_>,
    pos: horizon_core::position::BlockPos,
) {
    crate::game::ae2_energy_cell_item::flush_level(level, [pos.x, pos.y, pos.z]);
}
