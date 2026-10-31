fn ae2_guest_begin(
    hub: &crate::lan::hub::Hub,
    conn: u32,
    packet: &horizon_protocol::packets::play::ContainerClick,
    world: &crate::world::World,
) {
    crate::lan::menu::ae2_drive_context::begin(hub, conn, packet.container, world);
}
