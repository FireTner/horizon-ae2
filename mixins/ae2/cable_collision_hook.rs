fn ae2_cable_collision(
    scene: &crate::collisions::SceneCollisions<'_, '_>,
    pos: horizon_core::position::BlockPos,
) -> core::ops::ControlFlow<io::Result<VoxelShape>> {
    match crate::ae2_cable_view::collision(
        scene.world,
        scene.binding.registry(),
        [pos.x, pos.y, pos.z],
    ) {
        Some(shape) => core::ops::ControlFlow::Break(Ok(shape)),
        None => core::ops::ControlFlow::Continue(()),
    }
}
