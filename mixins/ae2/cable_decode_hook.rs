fn ae2_cable_decode(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    kind: i32,
    tag: &Tag,
) -> core::ops::ControlFlow<()> {
    let is_cable = game
        .remote
        .as_ref()
        .and_then(|r| r.builtin.get("minecraft:block_entity_type")?.name(kind))
        == Some("ae2:cable_bus");
    if !is_cable {
        return core::ops::ControlFlow::Continue(());
    }
    if game.native_block(pos) == "ae2:cable_bus"
        && let Some(be) = horizon_client::ae2_cable_sync::decode(tag)
    {
        game.world.block_entities.insert(
            (pos[0], pos[1], pos[2]),
            horizon_client::containers::BlockEntity::Mod(Box::new(be)),
        );
    }
    core::ops::ControlFlow::Break(())
}
