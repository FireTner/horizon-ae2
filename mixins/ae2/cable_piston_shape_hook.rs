fn ae2_cable_piston_shape(
    scene: &crate::collisions::SceneCollisions<'_, '_>,
    pos: [i32; 3],
) -> core::ops::ControlFlow<io::Result<VoxelShape>> {
    match crate::ae2_cable_view::collision(scene.world, scene.binding.registry(), pos) {
        Some(shape) => core::ops::ControlFlow::Break(Ok(shape)),
        None => core::ops::ControlFlow::Continue(()),
    }
}
