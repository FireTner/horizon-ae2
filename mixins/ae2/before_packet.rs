fn ae2_before_drive_packet(
    game: &mut crate::game::Game<'_>,
    packet: &horizon_protocol::packets::play::Serverbound,
) {
    if matches!(
        packet,
        horizon_protocol::packets::play::Serverbound::ContainerClick(_)
            | horizon_protocol::packets::play::Serverbound::ContainerClose(_)
    ) {
        crate::game::ae2_grid_glue::flush_drives(game);
    }
}
