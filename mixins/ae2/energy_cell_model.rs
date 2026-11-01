// SPDX-License-Identifier: LGPL-3.0-or-later
// Render-only projection for AE2's legacy fill_level predicate. Never changes
// the actual ItemStack, saved components, hashes or placement block-state data.
use super::{Context, ItemEnv, ItemModels, ResolvedItem, StackView, ae2_item_energy};
use serde_json::{Value, json};

fn projected(models: &ItemModels, stack: &StackView) -> Option<StackView> {
    let kind = ae2_item_energy::PoweredKind::from_item(&stack.item)?;
    if !kind.is_energy_cell() {
        return None;
    }
    let components = models.components(stack);
    let current = match components.get("ae2:stored_energy") {
        Some(Value::String(value)) if value == "NaN" => f64::NAN,
        Some(Value::String(value)) if value == "Infinity" => f64::INFINITY,
        Some(Value::String(value)) if value == "-Infinity" => f64::NEG_INFINITY,
        Some(value) => value.as_f64().unwrap_or(0.0),
        None => 0.0,
    };
    let level =
        ae2_item_energy::energy_cell_model_level(current, kind.default_capacity()).to_string();
    let mut state = components
        .get("minecraft:block_state")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if state.get("fullness").and_then(Value::as_str) == Some(level.as_str()) {
        return None;
    }
    state.insert("fullness".into(), json!(level));
    let mut projected = stack.clone();
    projected
        .patch
        .insert("minecraft:block_state".into(), Some(Value::Object(state)));
    Some(projected)
}

pub(super) fn append(
    models: &ItemModels,
    out: &mut ResolvedItem,
    stack: &StackView,
    ctx: &Context,
    env: &dyn ItemEnv,
) -> core::ops::ControlFlow<()> {
    let Some(projected) = projected(models, stack) else {
        return core::ops::ControlFlow::Continue(());
    };
    models.append_layers(out, &projected, ctx, env);
    core::ops::ControlFlow::Break(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_model::{ClientItem, LayerContent};
    struct Env;
    impl ItemEnv for Env {}

    #[test]
    fn actual_resolver_uses_fifths_without_mutating_stack_or_borrowing_capacity_override() {
        let mut models = ItemModels::default();
        for kind in [
            ae2_item_energy::PoweredKind::EnergyCell,
            ae2_item_energy::PoweredKind::DenseEnergyCell,
        ] {
            let id = if kind == ae2_item_energy::PoweredKind::EnergyCell {
                "ae2:energy_cell"
            } else {
                "ae2:dense_energy_cell"
            };
            let name = id.strip_prefix("ae2:").unwrap();
            let model = json!({"model":{"type":"minecraft:select", "property":"minecraft:block_state", "block_state_property":"fullness",
                "cases":(0..5).map(|i| json!({"when":i.to_string(),"model":{"type":"minecraft:model","model":format!("ae2:block/{name}_{i}")}})).collect::<Vec<_>>(),
                "fallback":{"type":"minecraft:model","model":format!("ae2:block/{name}_0")}}});
            models
                .definitions
                .insert(id.into(), ClientItem::decode(&model).unwrap());
            models.prototypes.insert(
                id.into(),
                [("minecraft:item_model".into(), json!(id))].into(),
            );
            for (ratio, level) in [
                (0.0, 0),
                (0.19999, 0),
                (0.2, 1),
                (0.4, 2),
                (0.6, 3),
                (0.8, 4),
                (1.0, 4),
            ] {
                let mut stack = StackView::new(id, 3);
                stack.patch.insert(
                    "ae2:stored_energy".into(),
                    Some(json!(ratio * kind.default_capacity())),
                );
                stack
                    .patch
                    .insert("ae2:energy_capacity".into(), Some(json!(1.0)));
                stack.patch.insert(
                    "minecraft:block_state".into(),
                    Some(json!({"foreign":"kept","fullness":"wrong"})),
                );
                let original = stack.clone();
                let projected = projected(&models, &stack).unwrap();
                assert_eq!(
                    projected.patch["minecraft:block_state"].as_ref().unwrap()["foreign"],
                    "kept"
                );
                let result = models.resolve(
                    &stack,
                    &Context {
                        display: super::super::DisplayContext::Gui,
                        owner: None,
                        level: None,
                        seed: 0,
                    },
                    &Env,
                );
                assert_eq!(stack, original);
                assert_eq!(result.layers.len(), 1);
                assert!(
                    matches!(&result.layers[0].content, LayerContent::Model { model } if model == &format!("ae2:block/{name}_{level}"))
                );
            }
            for (label, level) in [("NaN", 4), ("Infinity", 4), ("-Infinity", 0)] {
                let mut stack = StackView::new(id, 1);
                stack
                    .patch
                    .insert("ae2:stored_energy".into(), Some(json!(label)));
                let result = models.resolve(
                    &stack,
                    &Context {
                        display: super::super::DisplayContext::Gui,
                        owner: None,
                        level: None,
                        seed: 0,
                    },
                    &Env,
                );
                assert!(
                    matches!(&result.layers[0].content,LayerContent::Model{model} if model==&format!("ae2:block/{name}_{level}"))
                );
            }
        }
        assert!(projected(&models, &StackView::new("minecraft:stone", 1)).is_none());
    }
}
