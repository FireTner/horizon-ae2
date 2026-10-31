fn ae2_drive_open(
    game: &crate::game::Game<'_>,
    key: (i32, i32, i32),
    target: Option<(i32, i32, i32)>,
) {
    crate::game::ae2_gui::opened(game, key, target);
}
