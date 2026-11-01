fn ae2_energy_cell_placed(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
) {
    crate::game::ae2_energy_cell_item::placed(game, stack, pos);
}
