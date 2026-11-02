fn ae2_cable_prediction_ack(
    game: &mut crate::game::Game<'_>,
    sequence: i32,
) -> core::ops::ControlFlow<()> {
    thread_local! { static ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
    if ACTIVE.with(|a| a.replace(true)) {
        return core::ops::ControlFlow::Continue(());
    }
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            ACTIVE.with(|a| a.set(false));
        }
    }
    let _guard = Guard;
    game.end_predictions(sequence);
    ae2_cable_prediction::finish(game, sequence);
    core::ops::ControlFlow::Break(())
}
