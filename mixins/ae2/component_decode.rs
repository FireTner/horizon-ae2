// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_component_decode(
    id: &str,
    value: &horizon_world::nbt::Tag,
) -> core::ops::ControlFlow<Result<(), String>> {
    use horizon_world::nbt::Tag;
    if matches!(id, "ae2:stored_energy" | "ae2:energy_capacity") {
        return core::ops::ControlFlow::Break(
            if matches!(
                value,
                Tag::Byte(_)
                    | Tag::Short(_)
                    | Tag::Int(_)
                    | Tag::Long(_)
                    | Tag::Float(_)
                    | Tag::Double(_)
            ) {
                Ok(())
            } else {
                Err("Not a number".into())
            },
        );
    }
    core::ops::ControlFlow::Continue(())
}
