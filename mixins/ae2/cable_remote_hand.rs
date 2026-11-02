fn ae2_cable_remote_hand(
    game: &mut crate::game::Game<'_>,
    hand: Hand,
) -> core::ops::ControlFlow<bool> {
    if !game.with_hand(hand, |g| {
        g.held()
            .is_some_and(|s| crate::game::ae2_cable::Cable::from_item(&s.item).is_some())
    }) {
        return core::ops::ControlFlow::Continue(());
    }
    let Some(_scope) = crate::game::ae2_cable_actions::begin(game) else {
        return core::ops::ControlFlow::Continue(());
    };
    let used = game.remote_use_item_on(hand);
    core::ops::ControlFlow::Break(
        used || crate::game::ae2_cable_actions::result(game).is_some_and(|r| r.stops_click()),
    )
}
