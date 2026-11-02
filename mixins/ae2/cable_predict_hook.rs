fn ae2_cable_predict(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    hit: &crate::game::Hit,
) -> core::ops::ControlFlow<bool> {
    let Some(result) = ae2_cable_prediction::use_on(game, stack, hit) else {
        return core::ops::ControlFlow::Continue(());
    };
    crate::game::ae2_cable_actions::record(game, result);
    core::ops::ControlFlow::Break(result.swings())
}
