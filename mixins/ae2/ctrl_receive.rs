fn ae2_ctrl_receive(
    game: &mut crate::game::Game<'_>,
    host: &crate::game::lan::LanHost,
    conn: u32,
    data: &[u8],
) -> core::ops::ControlFlow<()> {
    let Ok(message) = mod_net::decode(data) else {
        return core::ops::ControlFlow::Continue(());
    };
    if message.channel != crate::game::ae2_ctrl::CHANNEL {
        return core::ops::ControlFlow::Continue(());
    }
    let Some(player) = host.hub.guest(conn).map(|guest| guest.uuid) else {
        return core::ops::ControlFlow::Break(());
    };
    // Preserve the original transport's per-player allowance, exactly once.
    let count = game.native_net.from_counts.entry(player).or_default();
    if *count < PER_TICK {
        *count += 1;
        crate::game::ae2_cable_input::receive(&game.world_root, player, &message.data);
    }
    core::ops::ControlFlow::Break(())
}
