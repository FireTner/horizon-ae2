fn ae2_ctrl_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::ae2_cable_input::key(game, key, pressed);
}
