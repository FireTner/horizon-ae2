fn ae2_cable_guest_packet(
    game: &mut crate::game::Game<'_>,
    host: &mut crate::game::lan::LanHost,
    conn: u32,
    packet: &S,
) -> core::ops::ControlFlow<()> {
    if !matches!(packet, S::UseItemOn(_)) {
        return core::ops::ControlFlow::Continue(());
    }
    let Some(_scope) = crate::game::ae2_cable_actions::packet_begin(game) else {
        return core::ops::ControlFlow::Continue(());
    };
    game.lan_guest_packet(host, conn, packet.clone());
    if let Some(kind) = host
        .hub
        .registry_id("minecraft:block_entity_type", "ae2:cable_bus")
    {
        for (pos, tag) in crate::game::ae2_cable_actions::take_updates(game) {
            host.hub.send_to(
                conn,
                &C::BlockEntityData(play::BlockEntityData { pos, kind, tag }),
            );
        }
    }
    core::ops::ControlFlow::Break(())
}
