// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_container(block: &str, container: &mut Option<horizon_client::containers::ModContainer>) {
    crate::game::ae2_glue::container(block, container);
}
