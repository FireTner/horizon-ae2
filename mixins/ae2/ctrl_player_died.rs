fn ae2_ctrl_player_died(game: &mut crate::game::Game<'_>, player: u128) {
    crate::game::ae2_cable_input::reset_player(&game.world_root, player);
}
