fn ae2_menu_place(
    menu: &crate::menu::Menu,
    slot: usize,
    stack: &crate::items::ItemStack,
) -> core::ops::ControlFlow<bool> {
    if crate::lan::menu::ae2_drive_context::client_drive()
        && menu.kind == crate::menu::Kind::Chest(2)
        && slot < 18
    {
        return core::ops::ControlFlow::Break(
            slot < 10 && crate::lan::menu::ae2_slot_rules::drive_slot(&stack.item, stack.count).0,
        );
    }
    core::ops::ControlFlow::Continue(())
}
