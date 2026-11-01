// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_component_send(
    item_id: &dyn Fn(&str) -> i32,
    stack: &Option<crate::items::ItemStack>,
) -> core::ops::ControlFlow<Option<Stack>> {
    if let Some(wire) = crate::item_sync::ae2_component_sync::encode(item_id, stack) {
        return core::ops::ControlFlow::Break(Some(wire));
    }
    core::ops::ControlFlow::Continue(())
}
