fn ae2_cable_loot(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
) -> core::ops::ControlFlow<Vec<ItemStack>> {
    if game.res.facts().get(state).name.as_ref() != "ae2:cable_bus" {
        return core::ops::ControlFlow::Continue(());
    }
    core::ops::ControlFlow::Break(if game.ui.rules.bool("block_drops") {
        crate::game::ae2_cable_items::remembered(&game.pending, pos)
    } else {
        vec![]
    })
}
