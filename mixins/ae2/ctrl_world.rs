fn ae2_ctrl_world(game: &mut crate::game::Game<'_>) {
    crate::game::ae2_cable_input::reset_world(&game.world_root);
}
