// SPDX-License-Identifier: LGPL-3.0-or-later
// ItemUpgradeInventory / UpgradeInventory / InitUpgrades, finite storage hosts.
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Host {
    ItemCell,
    FluidCell,
    PortableItem,
    PortableFluid,
    MatterCannon,
    ColorApplicator,
}
impl Host {
    pub fn slots(self) -> usize {
        match self {
            Self::FluidCell | Self::PortableFluid => 3,
            Self::ColorApplicator => 2,
            _ => 4,
        }
    }
    pub fn max_installed(self, card: &str) -> u32 {
        match card {
            "ae2:void_card" => 1,
            "ae2:equal_distribution_card" if self != Self::MatterCannon => 1,
            "ae2:inverter_card" if self != Self::ColorApplicator => 1,
            "ae2:fuzzy_card"
                if matches!(
                    self,
                    Self::ItemCell | Self::PortableItem | Self::MatterCannon
                ) =>
            {
                1
            }
            "ae2:energy_card"
                if matches!(
                    self,
                    Self::PortableItem
                        | Self::PortableFluid
                        | Self::MatterCannon
                        | Self::ColorApplicator
                ) =>
            {
                2
            }
            "ae2:speed_card" if self == Self::MatterCannon => 4,
            _ => 0,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Inventory {
    host: Host,
    slots: Vec<Option<String>>,
}
impl Inventory {
    pub fn new(host: Host, mut slots: Vec<Option<String>>) -> Self {
        slots.resize(host.slots(), None);
        slots.truncate(host.slots());
        Self { host, slots }
    }
    pub fn cards(&self) -> impl Iterator<Item = &str> {
        self.slots.iter().filter_map(Option::as_deref)
    }
    pub fn installed(&self) -> BTreeMap<&str, u32> {
        let mut counts = BTreeMap::new();
        for card in self.cards() {
            let max = self.host.max_installed(card);
            if max > 0 {
                let count = counts.entry(card).or_insert(0u32);
                *count = (*count + 1).min(max);
            }
        }
        counts
    }
    pub fn is_installed(&self, card: &str) -> bool {
        self.installed().contains_key(card)
    }
    // Upgrades.getEnergyCardMultiplier iterates cards, rather than using the
    // clamped installed count. Each built-in EnergyCardItem has multiplier 1.
    pub fn energy_multiplier(&self) -> u32 {
        self.cards().filter(|id| *id == "ae2:energy_card").count() as u32
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finite_host_tables_match_original_slot_limits_and_supported_cards() {
        assert_eq!(Host::FluidCell.slots(), 3);
        assert_eq!(Host::ColorApplicator.slots(), 2);
        assert_eq!(Host::MatterCannon.slots(), 4);
        assert_eq!(Host::FluidCell.max_installed("ae2:fuzzy_card"), 0);
        assert_eq!(Host::ItemCell.max_installed("ae2:energy_card"), 0);
        assert_eq!(Host::PortableItem.max_installed("ae2:energy_card"), 2);
        assert_eq!(Host::PortableFluid.max_installed("ae2:energy_card"), 2);
        assert_eq!(
            Host::MatterCannon.max_installed("ae2:equal_distribution_card"),
            0
        );
        assert_eq!(Host::ColorApplicator.max_installed("ae2:inverter_card"), 0);
        assert_eq!(Host::MatterCannon.max_installed("ae2:speed_card"), 4);
    }
    #[test]
    fn installed_counts_are_per_slot_clamped_and_distinct_from_energy_iteration() {
        let inv = Inventory::new(Host::PortableItem, vec![Some("ae2:energy_card".into()); 5]);
        assert_eq!(inv.cards().count(), 4);
        assert_eq!(inv.installed()["ae2:energy_card"], 2);
        assert_eq!(inv.energy_multiplier(), 4);
        let inv = Inventory::new(
            Host::ColorApplicator,
            vec![
                Some("ae2:fuzzy_card".into()),
                Some("ae2:void_card".into()),
                Some("ae2:equal_distribution_card".into()),
            ],
        );
        assert!(!inv.is_installed("ae2:fuzzy_card"));
        assert!(inv.is_installed("ae2:void_card"));
        assert!(!inv.is_installed("ae2:equal_distribution_card"));
    }
}
