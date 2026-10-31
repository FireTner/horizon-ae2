fn ae2_menu_max(menu: &crate::menu::Menu, slot: usize) -> core::ops::ControlFlow<u32> {
    if crate::lan::menu::ae2_drive_context::client_drive()
        && menu.kind == crate::menu::Kind::Chest(2)
        && slot < 10
    {
        return core::ops::ControlFlow::Break(1);
    }
    core::ops::ControlFlow::Continue(())
}
