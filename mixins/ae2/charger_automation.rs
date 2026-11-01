// SPDX-License-Identifier: LGPL-3.0-or-later
// Native ChargerInvFilter bridge. Ingredient membership is independent of
// whether this adapter can manufacture the recipe's result components.
use horizon_client::{
    crafting::{CustomRecipe, Ingredient},
    items::{ItemStack, Items},
};

pub(super) fn recipe_matches(recipes: &[CustomRecipe], items: &Items, stack: &ItemStack) -> bool {
    stack.count > 0
        && recipes.iter().any(|recipe| {
            super::ae2_glue::ingredient_json(&recipe.json["ingredient"])
                .and_then(|value| Ingredient::parse(&value, items))
                .is_some_and(|ingredient| ingredient.test(stack))
        })
}

fn matches(game: &super::Game<'_>, stack: &ItemStack) -> bool {
    game.recipes()
        .custom
        .get("ae2:charger")
        .is_some_and(|recipes| recipe_matches(recipes, game.items(), stack))
}

pub(super) fn allow_insert(game: &super::Game<'_>, stack: &ItemStack) -> bool {
    if stack.count == 0 {
        return false;
    }
    let Ok(powered) = super::ae2_item_energy_glue::load(stack) else {
        return false; // Retain malformed supported components, as in charging.
    };
    super::ae2_charger_automation_rules::allow_insert(
        powered.is_some_and(|item| item.is_chargeable()),
        matches(game, stack),
    )
}

pub(super) fn allow_extract(game: &super::Game<'_>, stack: &ItemStack) -> bool {
    // InternalInventoryItemHandler blocks internal wrapped stacks before
    // delegating to AppEngInternalInventory / ChargerInvFilter.
    if stack.count == 0 || &*stack.item == "ae2:wrapped_generic_stack" {
        return false;
    }
    let Ok(powered) = super::ae2_item_energy_glue::load(stack) else {
        return false;
    };
    let (chargeable, current, maximum) = powered.map_or((false, 0.0, 0.0), |item| {
        (
            item.is_chargeable(),
            item.energy.current(),
            item.energy.maximum(),
        )
    });
    super::ae2_charger_automation_rules::allow_extract(
        chargeable,
        current,
        maximum,
        matches(game, stack),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_world::{loot::PackOverlay, nbt::Tag};

    #[test]
    fn native_ingredient_membership_uses_tags_alternatives_and_ignores_result_support() {
        let root = std::path::PathBuf::from(std::env::var_os("HORIZON_RESOURCES").unwrap());
        let mut items = Items::catalog(&root, &PackOverlay::NONE).unwrap();
        items.item_tag_order.insert(
            "test:charger_input".into(),
            vec!["minecraft:iron_ingot".into()],
        );
        let recipes = vec![CustomRecipe {
            id: "test:component_result".into(),
            json: std::sync::Arc::new(serde_json::json!({
                "ingredient": [{"tag":"test:charger_input"}, {"item":"minecraft:gold_ingot"}],
                "result": {"id":"minecraft:book","components":{"example:unsupported":7}}
            })),
        }];
        for id in ["minecraft:iron_ingot", "minecraft:gold_ingot"] {
            let mut stack = ItemStack::new(id, 1);
            stack
                .data_mut()
                .set_other("example:data", Some(Tag::Long(123)));
            assert!(recipe_matches(&recipes, &items, &stack));
        }
        assert!(!recipe_matches(
            &recipes,
            &items,
            &ItemStack::new("minecraft:book", 1)
        ));
        assert!(!recipe_matches(
            &recipes,
            &items,
            &ItemStack::new("minecraft:iron_ingot", 0)
        ));
        items
            .item_tag_order
            .insert("test:charger_input".into(), vec![]);
        assert!(!recipe_matches(
            &recipes,
            &items,
            &ItemStack::new("minecraft:iron_ingot", 1)
        ));
    }

    #[test]
    fn native_power_predicates_use_current_components_and_preserve_loaded_overstack() {
        for id in [
            "ae2:charged_staff",
            "ae2:entropy_manipulator",
            "ae2:wireless_terminal",
            "ae2:wireless_crafting_terminal",
            "ae2:matter_cannon",
            "ae2:color_applicator",
        ]
        .into_iter()
        .map(str::to_owned)
        .chain(["item", "fluid"].into_iter().flat_map(|kind| {
            ["1k", "4k", "16k", "64k", "256k"]
                .into_iter()
                .map(move |tier| format!("ae2:portable_{kind}_cell_{tier}"))
        })) {
            let mut stack = ItemStack::new(&id, 3);
            let original = stack.clone();
            let powered = super::super::ae2_item_energy_glue::load(&stack)
                .unwrap()
                .unwrap();
            let maximum = powered.energy.maximum();
            assert!(super::super::ae2_charger_automation_rules::allow_extract(
                powered.is_chargeable(),
                powered.energy.current(),
                maximum,
                false
            ));
            assert!(!super::super::ae2_charger_automation_rules::allow_extract(
                powered.is_chargeable(),
                powered.energy.current(),
                maximum,
                true
            ));
            assert_eq!(stack, original);
            stack
                .data_mut()
                .set_other("ae2:stored_energy", Some(Tag::Double(maximum.to_bits())));
            let powered = super::super::ae2_item_energy_glue::load(&stack)
                .unwrap()
                .unwrap();
            assert!(super::super::ae2_charger_automation_rules::allow_extract(
                powered.is_chargeable(),
                powered.energy.current(),
                powered.energy.maximum(),
                true
            ));
            assert_eq!(stack.count, 3);
        }
    }
}
