fn ae2_menu_quick(
    menu: &mut crate::menu::Menu,
    env: &crate::menu::Env<'_>,
    index: usize,
) -> core::ops::ControlFlow<Option<ItemStack>> {
    if !crate::lan::menu::ae2_drive_context::client_drive()
        || menu.kind != crate::menu::Kind::Chest(2)
    {
        return core::ops::ControlFlow::Continue(());
    }
    // AEBaseMenu.quickMoveStack does not predict transfers on the client.
    if crate::lan::menu::ae2_drive_context::remote_window() {
        return core::ops::ControlFlow::Break(None);
    }
    let Some(mut stack) = menu.slot(index).cloned() else {
        return core::ops::ControlFlow::Break(None);
    };
    let targets = if index < 10 {
        Some((18, 54))
    } else if (18..54).contains(&index) {
        Some((0, 10))
    } else {
        None
    };
    if let Some((start, end)) = targets {
        while stack.count > 0 {
            let before = stack.count;
            menu.move_to(env, &mut stack, start, end, false);
            if stack.count == before {
                break;
            }
        }
        menu.put(env, index, (stack.count > 0).then_some(stack));
    }
    core::ops::ControlFlow::Break(None)
}
