// SPDX-License-Identifier: LGPL-3.0-or-later
// MatterCannonItem.isBlackListed/getPenetration and ColorApplicatorItem.getColorFrom.
// The adapter expands recipe ingredients and common dye tags from actual data.
use std::collections::BTreeSet;
pub const COLORS: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];
pub fn paintballs() -> BTreeSet<String> {
    COLORS
        .into_iter()
        .flat_map(|color| {
            [
                format!("ae2:{color}_paint_ball"),
                format!("ae2:{color}_lumen_paint_ball"),
            ]
        })
        .collect()
}
pub fn color_admission(tagged_dyes: impl IntoIterator<Item = String>) -> BTreeSet<String> {
    let mut accepted = paintballs();
    accepted.insert("minecraft:snowball".to_owned());
    accepted.extend(
        COLORS
            .into_iter()
            .map(|color| format!("minecraft:{color}_dye")),
    );
    accepted.extend(tagged_dyes);
    accepted
}
pub fn ammo_admission(
    recipes: impl IntoIterator<Item = (BTreeSet<String>, f32)>,
) -> BTreeSet<String> {
    let mut accepted = paintballs();
    let mut seen = BTreeSet::new();
    // The first matching recipe decides penetration, including zero/negative
    // weights. PaintBallItem is accepted regardless of recipe penetration.
    for (items, weight) in recipes {
        for item in items {
            if seen.insert(item.clone()) && weight > 0.0 {
                accepted.insert(item);
            }
        }
    }
    accepted
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn color_accepts_all_vanilla_dyes_paintballs_snowballs_and_tagged_additions() {
        let values = color_admission(["other:blue_dye".to_owned()]);
        assert_eq!(values.len(), 50);
        for color in COLORS {
            assert!(values.contains(&format!("minecraft:{color}_dye")));
            assert!(values.contains(&format!("ae2:{color}_paint_ball")));
            assert!(values.contains(&format!("ae2:{color}_lumen_paint_ball")));
        }
        assert!(values.contains("minecraft:snowball"));
        assert!(values.contains("other:blue_dye"));
        assert!(!values.contains("minecraft:water"));
        assert!(!values.contains("minecraft:stone"));
    }
    #[test]
    fn first_ammo_recipe_wins_and_paintballs_remain_valid_with_nonpositive_weights() {
        let set = |id: &str| BTreeSet::from([id.to_owned()]);
        let accepted = ammo_admission([
            (set("test:zero"), 0.0),
            (set("test:zero"), 10.0),
            (set("test:negative"), -1.0),
            (set("test:negative"), 10.0),
            (set("test:positive"), 1.0),
            (set("test:positive"), 0.0),
            (set("ae2:red_paint_ball"), -1.0),
        ]);
        assert!(!accepted.contains("test:zero"));
        assert!(!accepted.contains("test:negative"));
        assert!(accepted.contains("test:positive"));
        assert!(accepted.contains("ae2:red_paint_ball"));
        assert_eq!(accepted.len(), 33);
    }
}
