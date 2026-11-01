fn ae2_cable_outline(
    game: &crate::game::Game<'_>,
    pos: [i32; 3],
) -> core::ops::ControlFlow<Vec<[f64; 6]>> {
    match horizon_client_render::ae2_cable_view::outline(
        &game.world,
        &game.pack.assets.registry,
        pos,
    ) {
        Some(boxes) => core::ops::ControlFlow::Break(boxes),
        None => core::ops::ControlFlow::Continue(()),
    }
}
