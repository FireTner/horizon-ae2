fn ae2_guest_quick(
    view: &mut crate::lan::menu::View<'_>,
    slot: usize,
    rules: &dyn crate::lan::menu::Rules,
) -> core::ops::ControlFlow<()> {
    if !crate::lan::menu::ae2_drive_context::active(
        view.guest.uuid,
        view.guest.menu.as_ref().map(|m| m.window),
    ) || view.container.is_none()
    {
        return core::ops::ControlFlow::Continue(());
    }
    if view.enabled(slot)
        && let Some(stack) = view.get(slot)
    {
        let count = view.container.as_ref().unwrap().len();
        let rest = if slot < 10 {
            move_to(view, stack, count..count + 36, false, rules)
        } else if (count..count + 36).contains(&slot) {
            move_to(view, stack, 0..10, false, rules)
        } else {
            Some(stack)
        };
        view.set(slot, rest);
    }
    core::ops::ControlFlow::Break(())
}
