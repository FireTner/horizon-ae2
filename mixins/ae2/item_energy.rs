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
}
impl PoweredKind {
    pub fn from_item(id: &str) -> Option<Self> {
        match id {
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
    /// AEConfig defaults, before a native config supplier is connected.
    pub fn default_capacity(self) -> f64 {
        match self {
            Self::Portable | Self::ColorApplicator => 20000.0,
            Self::MatterCannon | Self::EntropyManipulator => 200000.0,
            Self::WirelessTerminal => 1600000.0,
            Self::ChargedStaff => 8000.0,
        }
    }
    pub fn charge_rate(self, energy_cards: u32) -> f64 {
        match self {
            Self::Portable | Self::ColorApplicator => 80.0 + 80.0 * energy_cards as f64,
            Self::MatterCannon | Self::WirelessTerminal => 800.0 + 800.0 * energy_cards as f64,
            Self::EntropyManipulator => 800.0,
            Self::ChargedStaff => 32.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemEnergy {
    default_capacity: f64,
    capacity_override: Option<f64>,
    stored: Option<f64>,
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
    pub fn set_current(&mut self, power: f64) {
        self.stored = if power < MIN_POWER { None } else { Some(power) };
    }
    /// Returns overflow, as IAEItemPowerStorage requires.
    pub fn inject(&mut self, amount: f64, commit: bool) -> f64 {
        let current = self.current();
        let required = self.maximum() - current;
        let overflow = java_max(0.0, java_min(amount - required, amount));
        if commit {
            self.set_current(current + java_min(amount, required));
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
#[cfg(test)]
mod tests {
    use super::*;
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
