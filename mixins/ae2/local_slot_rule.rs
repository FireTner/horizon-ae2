fn ae2_local_slot_rule(
    game: &crate::game::Game<'_>,
    slot: usize,
    stack: &horizon_client::items::ItemStack,
) -> core::ops::ControlFlow<(bool, u32)> {
    match crate::game::ae2_gui::slot_rule(game, slot, stack) {
        Some(rule) => core::ops::ControlFlow::Break(rule),
        None => core::ops::ControlFlow::Continue(()),
    }
}
