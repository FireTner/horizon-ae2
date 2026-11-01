// SPDX-License-Identifier: LGPL-3.0-or-later
// AEBasePoweredItem.java at upstream.lock.json; component energy rules only.
// Copyright (c) Applied Energistics 2 contributors; 2026 Horizon port contributors.

pub const MIN_POWER: f64 = 0.0001;

// Java Math.min/max propagate NaN and preserve the signed-zero ordering.
fn java_min(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        a
    } else if b.is_nan() {
        b
    } else if a == 0.0 && b == 0.0 {
        f64::from_bits(a.to_bits() | b.to_bits())
    } else if a <= b {
        a
    } else {
        b
    }
}
fn java_max(a: f64, b: f64) -> f64 {
    -java_min(-a, -b)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoweredKind {
    Portable,
    ColorApplicator,
    MatterCannon,
    WirelessTerminal,
    ChargedStaff,
    EntropyManipulator,
    EnergyCell,
    DenseEnergyCell,
}
impl PoweredKind {
    pub fn from_item(id: &str) -> Option<Self> {
        match id {
            "ae2:energy_cell" => Some(Self::EnergyCell),
            "ae2:dense_energy_cell" => Some(Self::DenseEnergyCell),
            "ae2:color_applicator" => Some(Self::ColorApplicator),
            "ae2:matter_cannon" => Some(Self::MatterCannon),
            "ae2:wireless_terminal" | "ae2:wireless_crafting_terminal" => {
                Some(Self::WirelessTerminal)
            }
            "ae2:charged_staff" => Some(Self::ChargedStaff),
            "ae2:entropy_manipulator" => Some(Self::EntropyManipulator),
            _ => {
                let tier = id
                    .strip_prefix("ae2:portable_item_cell_")
                    .or_else(|| id.strip_prefix("ae2:portable_fluid_cell_"))?;
                matches!(tier, "1k" | "4k" | "16k" | "64k" | "256k").then_some(Self::Portable)
            }
        }
    }
    pub fn is_energy_cell(self) -> bool {
        matches!(self, Self::EnergyCell | Self::DenseEnergyCell)
    }
    /// AEConfig defaults, before a native config supplier is connected.
    pub fn default_capacity(self) -> f64 {
        match self {
            Self::Portable | Self::ColorApplicator => 20000.0,
            Self::MatterCannon | Self::EntropyManipulator | Self::EnergyCell => 200000.0,
            Self::WirelessTerminal | Self::DenseEnergyCell => 1600000.0,
            Self::ChargedStaff => 8000.0,
        }
    }
    pub fn charge_rate(self, energy_cards: u32) -> f64 {
        match self {
            Self::Portable | Self::ColorApplicator => 80.0 + 80.0 * energy_cards as f64,
            Self::MatterCannon | Self::WirelessTerminal => 800.0 + 800.0 * energy_cards as f64,
            Self::EntropyManipulator => 800.0,
            Self::ChargedStaff => 32.0,
            Self::EnergyCell => 800.0,
            Self::DenseEnergyCell => 1600.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemEnergy {
    default_capacity: f64,
    capacity_override: Option<f64>,
    stored: Option<f64>,
    cell: bool,
}
impl ItemEnergy {
    /// Component loading does not silently clamp existing stored energy.
    /// The supplier's configured default is passed in by the host adapter.
    pub fn from_components(
        default_capacity: f64,
        capacity_override: Option<f64>,
        stored: Option<f64>,
    ) -> Self {
        Self {
            default_capacity,
            capacity_override,
            stored,
            cell: false,
        }
    }
    pub fn from_energy_cell(capacity: f64, stored: Option<f64>) -> Self {
        Self {
            default_capacity: capacity,
            capacity_override: None,
            stored,
            cell: true,
        }
    }
    pub fn capacity_override(&self) -> Option<f64> {
        self.capacity_override
    }
    pub fn stored_component(&self) -> Option<f64> {
        self.stored
    }
    pub fn maximum(&self) -> f64 {
        self.capacity_override.unwrap_or(self.default_capacity)
    }
    pub fn current(&self) -> f64 {
        self.stored.unwrap_or(0.0)
    }
    /// AEBasePoweredItem.getBarWidth: Java Math.round returns long, then the
    /// source narrows to int before clamping. Keep that order for huge values.
    pub fn bar_width(&self) -> i32 {
        let value = self.current() / self.maximum() * 13.0;
        let lower = value.floor();
        let rounded = if value - lower < 0.5 {
            lower as i64
        } else {
            (lower + 1.0) as i64
        };
        (rounded as i32).clamp(0, 13)
    }
    pub fn set_current(&mut self, power: f64) {
        self.stored = if power < if self.cell { 0.00001 } else { MIN_POWER } {
            None
        } else {
            Some(power)
        };
    }
    /// Returns overflow, as IAEItemPowerStorage requires.
    pub fn inject(&mut self, amount: f64, commit: bool) -> f64 {
        let current = self.current();
        let required = self.maximum() - current;
        let overflow = java_max(0.0, java_min(amount - required, amount));
        if commit {
            let added = if self.cell {
                java_min(required, amount)
            } else {
                java_min(amount, required)
            };
            self.set_current(current + added);
        }
        overflow
    }
    pub fn extract(&mut self, amount: f64, commit: bool) -> f64 {
        let current = self.current();
        let fulfillable = java_min(amount, current);
        if commit {
            self.set_current(current - fulfillable);
        }
        fulfillable
    }
    pub fn set_maximum(&mut self, maximum: f64) {
        self.capacity_override = if (maximum - self.default_capacity).abs() < MIN_POWER {
            None
        } else {
            Some(maximum)
        };
        if self.current() > maximum {
            self.set_current(maximum);
        }
    }
    pub fn set_maximum_multiplier(&mut self, multiplier: i32) {
        self.set_maximum(multiplier.clamp(1, 100) as f64 * self.default_capacity);
    }
    pub fn reset_maximum(&mut self) {
        self.set_maximum(self.default_capacity);
    }
}
/// InitItemModelsProperties uses a float fill ratio, and the original item
/// model overrides have thresholds .2/.4/.6/.8. This is NOT block fullness.
pub fn energy_cell_model_level(current: f64, maximum: f64) -> usize {
    let ratio = (current / maximum) as f32;
    [0.2f32, 0.4, 0.6, 0.8]
        .into_iter()
        .take_while(|threshold| ratio.partial_cmp(threshold) != Some(std::cmp::Ordering::Less))
        .count()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cell_item_uses_fixed_capacity_distinct_removal_threshold_and_item_model_ratios() {
        let mut cell = ItemEnergy::from_energy_cell(200000.0, Some(0.00005));
        let old = cell;
        assert_eq!(cell.extract(0.00001, false), 0.00001);
        assert_eq!(cell, old);
        cell.extract(0.00001, true);
        assert!(cell.stored_component().is_some());
        cell.extract(1.0, true);
        assert_eq!(cell.stored_component(), None);
        for (current, expected) in [
            (0.0, 0),
            (39999.0, 0),
            (40000.0, 1),
            (80000.0, 2),
            (120000.0, 3),
            (160000.0, 4),
            (200000.0, 4),
        ] {
            assert_eq!(energy_cell_model_level(current, 200000.0), expected);
        }
        assert_eq!(energy_cell_model_level(f64::NAN, 200000.0), 4);
        assert!(PoweredKind::EnergyCell.is_energy_cell());
        assert_eq!(PoweredKind::DenseEnergyCell.charge_rate(99), 1600.0);
    }
    #[test]
    fn energy_bar_is_thirteen_pixels_with_java_rounding_narrowing_and_nan_rules() {
        let mut energy = ItemEnergy::from_components(13.0, None, None);
        assert_eq!(energy.bar_width(), 0);
        energy.set_current(0.49999999999999994);
        assert_eq!(energy.bar_width(), 0);
        energy.set_current(0.5);
        assert_eq!(energy.bar_width(), 1);
        energy.set_current(12.5);
        assert_eq!(energy.bar_width(), 13);
        energy.set_current(26.0);
        assert_eq!(energy.bar_width(), 13);
        energy.set_current(f64::INFINITY);
        assert_eq!(energy.bar_width(), 0);
        energy.set_current(f64::NAN);
        assert_eq!(energy.bar_width(), 0);
    }
    #[test]
    fn injection_overflow_and_extraction_simulation_leave_components_untouched() {
        let mut energy = ItemEnergy::from_components(20000.0, None, Some(19990.0));
        let original = energy;
        assert_eq!(energy.inject(80.0, false), 70.0);
        assert_eq!(energy.extract(80.0, false), 80.0);
        assert_eq!(energy, original);
        assert_eq!(energy.inject(80.0, true), 70.0);
        assert_eq!(energy.current(), 20000.0);
        assert_eq!(energy.extract(50000.0, true), 20000.0);
        assert_eq!(energy.stored_component(), None);
    }
    #[test]
    fn threshold_and_capacity_normalization_preserve_original_boundaries() {
        let mut energy = ItemEnergy::from_components(20000.0, None, None);
        energy.set_current(MIN_POWER);
        assert_eq!(energy.stored_component(), Some(MIN_POWER));
        energy.set_current(MIN_POWER / 2.0);
        assert_eq!(energy.stored_component(), None);
        energy.set_maximum_multiplier(200);
        assert_eq!(energy.maximum(), 2000000.0);
        energy.set_current(1500000.0);
        energy.set_maximum_multiplier(-5);
        assert_eq!(energy.maximum(), 20000.0);
        assert_eq!(energy.capacity_override(), None);
        assert_eq!(energy.current(), 20000.0);
        energy.set_maximum(20000.0 + MIN_POWER / 2.0);
        assert_eq!(energy.capacity_override(), None);
        energy.set_maximum(20000.0 + MIN_POWER * 2.0);
        assert!(energy.capacity_override().is_some());
        energy.reset_maximum();
        assert_eq!(energy.capacity_override(), None);
    }
    #[test]
    fn loaded_over_capacity_values_are_only_clamped_by_mutating_operations() {
        let mut energy = ItemEnergy::from_components(100.0, None, Some(120.0));
        assert_eq!(energy.current(), 120.0);
        assert_eq!(energy.inject(10.0, false), 10.0);
        assert_eq!(energy.current(), 120.0);
        assert_eq!(energy.inject(10.0, true), 10.0);
        assert_eq!(energy.current(), 100.0);
        // Source does not validate the sign of an extraction request.
        assert_eq!(energy.extract(-5.0, true), -5.0);
        assert_eq!(energy.current(), 105.0);
    }
    #[test]
    fn floating_point_minimum_and_nan_follow_java_operations() {
        assert_eq!(java_min(0.0, -0.0).to_bits(), (-0.0f64).to_bits());
        assert_eq!(java_max(0.0, -0.0).to_bits(), 0.0f64.to_bits());
        let mut energy = ItemEnergy::from_components(100.0, None, None);
        assert!(energy.inject(f64::NAN, false).is_nan());
        assert_eq!(energy.stored_component(), None);
        energy.inject(f64::NAN, true);
        assert!(energy.current().is_nan());
    }
}
