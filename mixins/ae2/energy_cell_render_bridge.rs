// SPDX-License-Identifier: LGPL-3.0-or-later
// JSON cannot carry IEEE nonfinite numbers. Preserve these three numeric cases
// only in the transient renderer view of the two finite Energy Cell items.
use horizon_world::nbt::Tag;
use std::cell::Cell;
thread_local! { static PROJECTING: Cell<bool> = const { Cell::new(false) }; }
struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        PROJECTING.with(|flag| flag.set(false));
    }
}
pub(super) fn view(
    stack: &crate::items::ItemStack,
) -> core::ops::ControlFlow<horizon_assets::item_model::StackView> {
    if PROJECTING.with(Cell::get)
        || !matches!(&*stack.item, "ae2:energy_cell" | "ae2:dense_energy_cell")
    {
        return core::ops::ControlFlow::Continue(());
    }
    let value = match stack
        .data()
        .and_then(|data| data.other("ae2:stored_energy"))
    {
        Some(Tag::Double(bits)) => f64::from_bits(*bits),
        Some(Tag::Float(bits)) => f32::from_bits(*bits) as f64,
        _ => return core::ops::ControlFlow::Continue(()),
    };
    if value.is_finite() {
        return core::ops::ControlFlow::Continue(());
    }
    PROJECTING.with(|flag| flag.set(true));
    let _guard = Guard;
    let mut projected = super::stack_view(stack);
    let label = if value.is_nan() {
        "NaN"
    } else if value.is_sign_positive() {
        "Infinity"
    } else {
        "-Infinity"
    };
    projected.patch.insert(
        "ae2:stored_energy".into(),
        Some(serde_json::Value::String(label.into())),
    );
    core::ops::ControlFlow::Break(projected)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_stack_view_preserves_nonfinite_energy_for_rendering_without_changing_real_components()
    {
        for (value, label) in [
            (f64::NAN, "NaN"),
            (f64::INFINITY, "Infinity"),
            (f64::NEG_INFINITY, "-Infinity"),
        ] {
            let mut stack = crate::items::ItemStack::new("ae2:energy_cell", 2);
            stack
                .data_mut()
                .set_other("ae2:stored_energy", Some(Tag::Double(value.to_bits())));
            let before = stack.clone();
            let view = super::super::stack_view(&stack);
            assert_eq!(
                view.patch["ae2:stored_energy"],
                Some(serde_json::json!(label))
            );
            assert_eq!(stack, before);
        }
    }
}
