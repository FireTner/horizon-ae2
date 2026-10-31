// SPDX-License-Identifier: LGPL-3.0-or-later
// Native recipe/tag expansion for Matter Cannon and Color Applicator inventories.
use super::{Game, ae2_tool_filters as rules};
use horizon_client::{crafting::Ingredient, items::Items};
use serde_json::Value;
use std::collections::BTreeSet;

fn conditions_pass(value: Option<&Value>, items: &Items) -> bool {
    fn test(value: &Value, items: &Items) -> Option<bool> {
        match value["type"].as_str() {
            Some("neoforge:not") => test(value.get("value")?, items).map(|value| !value),
            Some("neoforge:tag_empty") => value["tag"]
                .as_str()
                .map(|tag| items.item_tags.get(tag).is_none_or(|set| set.is_empty())),
            // The pinned ammo data uses only not(tag_empty). Other loaders'
            // condition types require their own native adapters.
            _ => None,
        }
    }
    value.is_none_or(|value| {
        value
            .as_array()
            .is_some_and(|values| values.iter().all(|v| test(v, items) == Some(true)))
    })
}
fn from_data<'a>(
    items: &Items,
    id: &str,
    recipes: impl Iterator<Item = &'a serde_json::Map<String, Value>>,
) -> Option<BTreeSet<String>> {
    match id {
        "ae2:color_applicator" => Some(rules::color_admission(rules::COLORS.into_iter().flat_map(
            |color| {
                items
                    .item_tags
                    .get(&format!("c:dyes/{color}"))
                    .into_iter()
                    .flatten()
                    .cloned()
            },
        ))),
        "ae2:matter_cannon" => Some(rules::ammo_admission(recipes.filter_map(|recipe| {
            if !conditions_pass(recipe.get("neoforge:conditions"), items) {
                return None;
            }
            let value = super::ae2_glue::ingredient_json(recipe.get("ammo")?)?;
            let ingredient = Ingredient::parse(&value, items)?;
            let weight = recipe.get("weight")?.as_f64()? as f32;
            Some((ingredient.0.into_iter().collect(), weight))
        }))),
        _ => None,
    }
}
pub(super) fn accepted(game: &Game<'_>, id: &str) -> Option<BTreeSet<String>> {
    from_data(
        game.items(),
        id,
        game.recipes()
            .custom
            .get("ae2:matter_cannon")
            .into_iter()
            .flatten()
            .filter_map(|recipe| recipe.json.as_object()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_world::loot::PackOverlay;
    fn items() -> Items {
        let root = std::path::PathBuf::from(
            std::env::var_os("HORIZON_RESOURCES")
                .expect("tool recipe/tag tests require HORIZON_RESOURCES"),
        );
        Items::catalog(&root, &PackOverlay::NONE).unwrap()
    }
    #[test]
    fn native_ammo_ingredients_conditions_and_first_match_use_real_items_catalog() {
        let mut items = items();
        items.item_tags.insert(
            "test:ammo".into(),
            std::collections::HashSet::from(["minecraft:iron_nugget".into()]),
        );
        items
            .item_tag_order
            .insert("test:ammo".into(), vec!["minecraft:iron_nugget".into()]);
        let recipes = [
            serde_json::json!({"ammo":{"tag":"test:ammo"},"weight":0.0,"neoforge:conditions":[{"type":"neoforge:not","value":{"type":"neoforge:tag_empty","tag":"test:ammo"}}]}),
            serde_json::json!({"ammo":{"item":"minecraft:iron_nugget"},"weight":50.0}),
            serde_json::json!({"ammo":{"item":"minecraft:gold_nugget"},"weight":50.0,"neoforge:conditions":[{"type":"neoforge:not","value":{"type":"neoforge:tag_empty","tag":"test:missing"}}]}),
            serde_json::json!({"ammo":{"item":"ae2:matter_ball"},"weight":32.0}),
            serde_json::json!({"ammo":{"item":"minecraft:copper_ingot"},"weight":32.0,"neoforge:conditions":[{"type":"neoforge:not","value":{"type":"other:unknown"}}]}),
        ];
        let accepted = from_data(
            &items,
            "ae2:matter_cannon",
            recipes.iter().map(|recipe| recipe.as_object().unwrap()),
        )
        .unwrap();
        assert!(!accepted.contains("minecraft:iron_nugget"));
        assert!(!accepted.contains("minecraft:gold_nugget"));
        assert!(!accepted.contains("minecraft:copper_ingot"));
        assert!(accepted.contains("ae2:matter_ball"));
        assert!(accepted.contains("ae2:red_paint_ball"));
    }
    #[test]
    fn native_color_filter_uses_common_dye_tags_and_ignores_recipe_data() {
        let mut items = items();
        items.item_tags.insert(
            "c:dyes/red".into(),
            std::collections::HashSet::from(["minecraft:iron_ingot".into()]),
        );
        let accepted = from_data(&items, "ae2:color_applicator", std::iter::empty()).unwrap();
        assert!(accepted.contains("minecraft:iron_ingot"));
        assert!(accepted.contains("minecraft:red_dye"));
        assert!(accepted.contains("minecraft:snowball"));
        assert!(!accepted.contains("minecraft:stone"));
        assert!(from_data(&items, "minecraft:stone", std::iter::empty()).is_none());
    }
}
