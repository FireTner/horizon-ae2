fn ae2_cable_geometry(
    game: &mut crate::game::Game<'_>,
    camera: glam::DVec3,
    out: &mut Vec<horizon_client::mesher::Vertex>,
) {
    horizon_client_render::ae2_cable_view::geometry(
        &game.world,
        &game.pack.assets.registry,
        camera,
        &game.gui.atlas.sprites,
        out,
    );
}
