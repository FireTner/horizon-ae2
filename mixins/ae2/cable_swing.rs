fn ae2_cable_swing(game: &crate::game::Game<'_>) -> core::ops::ControlFlow<()> {
    if crate::game::ae2_cable_actions::result(game).is_some_and(|r| !r.swings()) {
        core::ops::ControlFlow::Break(())
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
