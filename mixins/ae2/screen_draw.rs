fn ae2_drive_draw(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    x: f32,
    y: f32,
) -> core::ops::ControlFlow<bool> {
    if crate::game::ae2_gui::draw_full(game, batch, [x, y]) {
        core::ops::ControlFlow::Break(true)
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
