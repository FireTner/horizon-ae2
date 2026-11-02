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
    if let Some(be) = horizon_client::ae2_cable_sync::decode(tag) {
        let entity = horizon_client::containers::BlockEntity::Mod(Box::new(be));
        ae2_cable_prediction::verified(game, pos, &entity);
        if game.native_block(pos) == "ae2:cable_bus" {
            game.world
                .block_entities
                .insert((pos[0], pos[1], pos[2]), entity);
        }
    }
    core::ops::ControlFlow::Break(())
}
