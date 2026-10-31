fn ae2_automation_insert(
    game: &mut crate::game::Game<'_>,
    view: &crate::game::machines::View,
    slot: usize,
    stack: &horizon_client::items::ItemStack,
) -> core::ops::ControlFlow<Option<ItemStack>> {
    let Some((pos, index)) = view.locate(slot) else {
        return core::ops::ControlFlow::Continue(());
    };
    if !matches!(game.world.block_entities.get(&pos), Some(horizon_client::containers::BlockEntity::Mod(be)) if be.id == "ae2:drive")
    {
        return core::ops::ControlFlow::Continue(());
    }
    if index >= 10 {
        return core::ops::ControlFlow::Break(Some(stack.clone()));
    }
    let Some(target) = game.slot_mut_at(view, slot) else {
        return core::ops::ControlFlow::Break(Some(stack.clone()));
    };
    let (rest, changed) = crate::game::ae2_automation::insert(target, stack.clone());
    if changed {
        game.view_changed(view);
    }
    core::ops::ControlFlow::Break(rest)
}
