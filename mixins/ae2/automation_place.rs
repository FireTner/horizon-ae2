fn ae2_automation_place(
    containers: &crate::game::machines::Containers<'_>,
    view: &crate::game::machines::View,
    slot: usize,
    stack: &horizon_client::items::ItemStack,
) -> core::ops::ControlFlow<bool> {
    if let Some((pos, index)) = view.locate(slot)
        && matches!(containers.world.block_entities.get(&pos), Some(horizon_client::containers::BlockEntity::Mod(be)) if be.id == "ae2:drive")
    {
        return core::ops::ControlFlow::Break(
            index < 10
                && horizon_client::lan::menu::ae2_slot_rules::drive_slot(&stack.item, stack.count)
                    .0,
        );
    }
    core::ops::ControlFlow::Continue(())
}
