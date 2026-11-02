fn ae2_cable_hand(
    game: &mut crate::game::Game<'_>,
    hit: Option<&crate::game::Hit>,
) -> core::ops::ControlFlow<Result<bool>> {
    if hit.is_none()
        || !game
            .held()
            .is_some_and(|s| crate::game::ae2_cable::Cable::from_item(&s.item).is_some())
    {
        return core::ops::ControlFlow::Continue(());
    }
    let Some(_scope) = crate::game::ae2_cable_actions::begin(game) else {
        return core::ops::ControlFlow::Continue(());
    };
    core::ops::ControlFlow::Break(game.use_hand(hit))
}
