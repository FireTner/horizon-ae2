fn ae2_cable_prediction_break(game: &crate::game::Game<'_>, pos: [i32; 3], sequence: i32) {
    if game.native_block(pos) == "ae2:cable_bus" {
        ae2_cable_prediction::retain(game, pos, sequence);
    }
}
