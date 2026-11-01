// SPDX-License-Identifier: LGPL-3.0-or-later
// Derived from AE2 19.2.18 ChargerBlockEntity.ChargerInvFilter / ChargerRecipes.
// Copyright (c) Applied Energistics 2 contributors; 2026 Horizon port contributors.

pub fn allow_insert(chargeable: bool, recipe_matches: bool) -> bool {
    chargeable || recipe_matches
}

pub fn allow_extract(
    chargeable: bool,
    current_power: f64,
    maximum_power: f64,
    recipe_matches: bool,
) -> bool {
    (chargeable && current_power >= maximum_power) || !recipe_matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_absence_allows_even_unfinished_chargeable_items() {
        for chargeable in [false, true] {
            assert!(allow_extract(chargeable, 0.0, 100.0, false));
            assert_eq!(allow_insert(chargeable, false), chargeable);
            assert!(allow_insert(chargeable, true));
        }
    }

    #[test]
    fn recipe_ingredients_wait_unless_chargeable_and_full() {
        assert!(!allow_extract(false, 100.0, 100.0, true));
        assert!(!allow_extract(true, 99.999, 100.0, true));
        assert!(allow_extract(true, 100.0, 100.0, true));
        assert!(allow_extract(true, 101.0, 100.0, true));
    }

    #[test]
    fn comparisons_follow_java_for_nonfinite_values_and_signed_zero() {
        assert!(!allow_extract(true, f64::NAN, 1.0, true));
        assert!(!allow_extract(true, 1.0, f64::NAN, true));
        assert!(allow_extract(true, f64::INFINITY, f64::INFINITY, true));
        assert!(allow_extract(true, -0.0, 0.0, true));
        assert!(allow_extract(true, f64::NAN, f64::NAN, false));
    }
}
