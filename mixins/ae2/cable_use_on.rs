fn ae2_cable_use_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    if let Some(result) = crate::game::ae2_cable_items::use_on(game, stack, pos, face, claimed) {
        crate::game::ae2_cable_actions::record(game, result);
    }
}
