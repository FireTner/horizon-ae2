// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_block_entity(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::ae2_glue::block_entity(block, entity);
}
