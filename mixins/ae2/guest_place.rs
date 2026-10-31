fn ae2_guest_place(
    view: &crate::lan::menu::View<'_>,
    slot: usize,
    stack: &crate::items::ItemStack,
) -> core::ops::ControlFlow<bool> {
    if crate::lan::menu::ae2_drive_context::active(
        view.guest.uuid,
        view.guest.menu.as_ref().map(|m| m.window),
    ) && view.container.as_ref().is_some_and(|c| slot < c.len())
    {
        return core::ops::ControlFlow::Break(
            view.enabled(slot)
                && slot < 10
                && crate::lan::menu::ae2_slot_rules::drive_slot(&stack.item, stack.count).0,
        );
    }
    core::ops::ControlFlow::Continue(())
}
