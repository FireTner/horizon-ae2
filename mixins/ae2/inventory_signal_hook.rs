fn ae2_inventory_signal_override(
    containers: &crate::game::machines::Containers<'_>,
    view: &crate::game::machines::View,
) -> core::ops::ControlFlow<i32> {
    if let Some((pos, _)) = view.locate(0)
        && let Some(horizon_client::containers::BlockEntity::Mod(be)) =
            containers.world.block_entities.get(&pos)
        && let Some(signal) = super::ae2_inventory_signal_glue::signal(&be.id, &be.nbt, &be.items)
    {
        return core::ops::ControlFlow::Break(signal);
    }
    core::ops::ControlFlow::Continue(())
}
