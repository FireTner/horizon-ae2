// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_component_registry(game: &mut crate::game::Game<'_>) {
    // build_registries hydrates persistent types only while the list is empty.
    // Seed that vanilla list before adding AE2 types, so the original conditional
    // hydration cannot accidentally skip all vanilla components.
    let types = "minecraft:data_component_type";
    if game.ui.registries.persistent_components.is_empty()
        || !game.ui.registries.by_registry.contains_key(types)
    {
        for dir in [&game.options.resources, &game.ui.vanilla_root] {
            if let Some(export) = command_export(dir) {
                let mut vanilla = horizon_client::command::Registries::default();
                vanilla.fill_from_export(&export);
                if game.ui.registries.persistent_components.is_empty() {
                    game.ui.registries.persistent_components = vanilla.persistent_components;
                }
                if !game.ui.registries.by_registry.contains_key(types)
                    && let Some(ids) = vanilla.by_registry.remove(types)
                {
                    game.ui.registries.by_registry.insert(types.to_owned(), ids);
                }
                break;
            }
        }
    }
    crate::game::ae2_item_energy_glue::register_components(&mut game.ui.registries);
}
