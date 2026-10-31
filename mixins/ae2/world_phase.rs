// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_world_phase() {
    crate::game::ae2_grid_glue::reset();
    horizon_client::lan::menu::ae2_drive_context::reset();
    crate::game::ae2_gui::closed();
}
