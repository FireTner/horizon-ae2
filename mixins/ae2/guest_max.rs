fn ae2_guest_max(view: &crate::lan::menu::View<'_>, slot: usize) -> core::ops::ControlFlow<u32> {
    if crate::lan::menu::ae2_drive_context::active(
        view.guest.uuid,
        view.guest.menu.as_ref().map(|m| m.window),
    ) && view.container.as_ref().is_some_and(|c| slot < c.len())
    {
        return core::ops::ControlFlow::Break(if slot < 10 { 1 } else { 0 });
    }
    core::ops::ControlFlow::Continue(())
}
