// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_component_receive(
    stack: &horizon_protocol::item::Stack,
    builtin: &horizon_protocol::registry::Builtin,
) -> core::ops::ControlFlow<Option<Tag>> {
    if let Some(components) = crate::item_sync::ae2_component_sync::decode(stack, builtin) {
        return core::ops::ControlFlow::Break(Some(components));
    }
    core::ops::ControlFlow::Continue(())
}
