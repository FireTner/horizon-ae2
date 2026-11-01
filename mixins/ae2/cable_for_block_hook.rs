fn ae2_cable_for_block(
    game: &mut crate::game::Game<'_>,
    pos: (i32, i32, i32),
    state: horizon_world::registry::BlockStateId,
) -> core::ops::ControlFlow<()> {
    use horizon_client::containers::{BlockEntity, ModBlockEntity};
    if game.res.facts().get(state).name.as_ref() != "ae2:cable_bus" {
        return core::ops::ControlFlow::Continue(());
    }
    if !matches!(game.world.block_entities.get(&pos),Some(BlockEntity::Mod(be)) if be.id=="ae2:cable_bus")
    {
        game.world.block_entities.insert(
            pos,
            BlockEntity::Mod(Box::new(ModBlockEntity {
                id: "ae2:cable_bus".into(),
                nbt: Default::default(),
                items: Vec::new(),
                data: Vec::new(),
            })),
        );
    }
    core::ops::ControlFlow::Break(())
}
