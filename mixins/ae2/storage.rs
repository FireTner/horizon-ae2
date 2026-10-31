// SPDX-License-Identifier: LGPL-3.0-or-later
// BasicCellInventory, BasicStorageCell and StorageTier (see upstream.lock.json).
// Copyright (c) 2013-2021 AlgorithmX2 / TeamAppliedEnergistics;
// Copyright (c) 2026 Horizon port contributors.
// Rules only: the adapter owns AEKey components, fuzzy matching and persistence.

pub use super::ae2_keys::{FuzzyMode, KeyType};
use std::collections::{BTreeMap, BTreeSet};

/// Key equality MUST include all data components, not just the registry ID.
pub trait CellKey: Ord + Clone {
    fn key_type(&self) -> KeyType;
    fn can_fit_inside_cell(&self) -> bool {
        true
    }
    fn blacklisted(&self) -> bool {
        false
    }
    fn fuzzy_search_matches(&self, candidate: &Self, _mode: FuzzyMode) -> bool {
        self == candidate
    }
}

impl CellKey for super::ae2_keys::Key {
    fn key_type(&self) -> KeyType {
        self.kind()
    }
    fn can_fit_inside_cell(&self) -> bool {
        self.properties().can_fit_inside_cell
    }
    fn blacklisted(&self) -> bool {
        self.properties().blacklisted
    }
    fn fuzzy_search_matches(&self, candidate: &Self, mode: FuzzyMode) -> bool {
        self.fuzzy_search_matches(candidate, mode)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    K1,
    K4,
    K16,
    K64,
    K256,
}

impl Tier {
    pub const ALL: [Self; 5] = [Self::K1, Self::K4, Self::K16, Self::K64, Self::K256];
    pub fn bytes(self) -> u64 {
        match self {
            Self::K1 => 1024,
            Self::K4 => 4096,
            Self::K16 => 16384,
            Self::K64 => 65536,
            Self::K256 => 262144,
        }
    }
    pub fn bytes_per_type(self) -> u64 {
        self.bytes() / 128
    }
    pub fn idle_drain(self) -> f64 {
        match self {
            Self::K1 => 0.5,
            Self::K4 => 1.0,
            Self::K16 => 1.5,
            Self::K64 => 2.0,
            Self::K256 => 2.5,
        }
    }
}

/// IBasicCellItem's capacity/type/drain rules. Portable cells use the same
/// inventory algorithm with a smaller capacity and a tier-specific type limit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellLayout {
    bytes: u64,
    bytes_per_type: u64,
    total_types: usize,
    idle_drain: f64,
}
impl CellLayout {
    pub fn basic(tier: Tier, kind: KeyType) -> Self {
        Self {
            bytes: tier.bytes(),
            bytes_per_type: tier.bytes_per_type(),
            total_types: kind.total_types(),
            idle_drain: tier.idle_drain(),
        }
    }
    pub fn portable(tier: Tier, kind: KeyType) -> Self {
        let index = match tier {
            Tier::K1 => 1,
            Tier::K4 => 2,
            Tier::K16 => 3,
            Tier::K64 => 4,
            Tier::K256 => 5,
        };
        Self {
            bytes: tier.bytes() / 2,
            bytes_per_type: tier.bytes_per_type(),
            total_types: if kind == KeyType::Item {
                63 - index * 9
            } else {
                18
            },
            idle_drain: 0.5,
        }
    }
    pub fn bytes(self) -> u64 {
        self.bytes
    }
    pub fn total_types(self) -> usize {
        self.total_types
    }
    pub fn bytes_per_type(self) -> u64 {
        self.bytes_per_type
    }
    pub fn idle_drain(self) -> f64 {
        self.idle_drain
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {
    Empty,
    NotEmpty,
    TypesFull,
    Full,
}

#[derive(Clone, Debug)]
pub struct Cell<K> {
    tier: Tier,
    kind: KeyType,
    layout: CellLayout,
    stored: BTreeMap<K, u64>,
    partition: BTreeSet<K>,
    inverted: bool,
    equal_distribution: bool,
    void: bool,
    fuzzy: Option<FuzzyMode>,
}

impl<K: CellKey> Cell<K> {
    pub fn new(tier: Tier, kind: KeyType) -> Self {
        Self::new_with_layout(tier, kind, CellLayout::basic(tier, kind))
    }
    pub fn new_with_layout(tier: Tier, kind: KeyType, layout: CellLayout) -> Self {
        Self {
            tier,
            kind,
            layout,
            stored: BTreeMap::new(),
            partition: BTreeSet::new(),
            inverted: false,
            equal_distribution: false,
            void: false,
            fuzzy: None,
        }
    }

    /// Partition keys. set_fuzzy enables FuzzyPriorityList range semantics.
    pub fn configure(
        &mut self,
        partition: BTreeSet<K>,
        inverted: bool,
        equal_distribution: bool,
        void: bool,
    ) {
        self.partition = partition;
        self.inverted = inverted;
        self.equal_distribution = equal_distribution;
        self.void = void;
    }

    pub fn set_fuzzy(&mut self, fuzzy: Option<FuzzyMode>) {
        self.fuzzy = fuzzy;
    }

    /// Load without reinserting through current filters/upgrades (which may
    /// have changed). Invalid content is returned to the adapter as an error;
    /// the adapter must keep the original component unchanged, never truncate.
    pub fn restore(
        tier: Tier,
        kind: KeyType,
        stacks: BTreeMap<K, u64>,
    ) -> Result<Self, &'static str> {
        Self::restore_with_layout(tier, kind, CellLayout::basic(tier, kind), stacks)
    }
    pub fn restore_with_layout(
        tier: Tier,
        kind: KeyType,
        layout: CellLayout,
        stacks: BTreeMap<K, u64>,
    ) -> Result<Self, &'static str> {
        if stacks.len() > layout.total_types {
            return Err("cell type limit exceeded");
        }
        let mut total = 0u64;
        for (key, count) in &stacks {
            if key.key_type() != kind || *count == 0 {
                return Err("invalid cell entry");
            }
            total = total.checked_add(*count).ok_or("cell count overflow")?;
        }
        let used = (stacks.len() as u64)
            .checked_mul(layout.bytes_per_type)
            .and_then(|v| v.checked_add(total.div_ceil(kind.amount_per_byte())))
            .ok_or("cell bytes overflow")?;
        if used > layout.bytes {
            return Err("cell capacity exceeded");
        }
        let mut result = Self::new_with_layout(tier, kind, layout);
        result.stored = stacks;
        Ok(result)
    }

    pub fn stacks(&self) -> &BTreeMap<K, u64> {
        &self.stored
    }
    pub fn tier(&self) -> Tier {
        self.tier
    }
    pub fn kind(&self) -> KeyType {
        self.kind
    }
    pub fn layout(&self) -> CellLayout {
        self.layout
    }
    pub fn is_preformatted(&self) -> bool {
        !self.partition.is_empty()
    }
    pub fn is_inverted(&self) -> bool {
        self.inverted
    }
    pub fn fuzzy_mode(&self) -> Option<FuzzyMode> {
        self.fuzzy
    }
    pub fn stored_types(&self) -> usize {
        self.stored.len()
    }
    pub fn stored_count(&self) -> u64 {
        self.stored.values().sum()
    }
    pub fn unused_count(&self) -> u64 {
        let per = self.kind.amount_per_byte();
        (per - self.stored_count() % per) % per
    }
    pub fn used_bytes(&self) -> u64 {
        self.stored_types() as u64 * self.layout.bytes_per_type
            + (self.stored_count() + self.unused_count()) / self.kind.amount_per_byte()
    }
    pub fn free_bytes(&self) -> u64 {
        self.layout.bytes - self.used_bytes()
    }
    pub fn remaining_types(&self) -> u64 {
        (self.free_bytes() / self.layout.bytes_per_type)
            .min((self.layout.total_types - self.stored_types()) as u64)
    }
    pub fn can_hold_new_type(&self) -> bool {
        let free = self.free_bytes();
        (free > self.layout.bytes_per_type
            || free == self.layout.bytes_per_type && self.unused_count() > 0)
            && self.remaining_types() > 0
    }
    pub fn remaining_count(&self) -> u64 {
        self.free_bytes() * self.kind.amount_per_byte() + self.unused_count()
    }
    pub fn state(&self) -> CellState {
        if self.stored.is_empty() {
            CellState::Empty
        } else if self.can_hold_new_type() {
            CellState::NotEmpty
        } else if self.remaining_count() > 0 {
            CellState::TypesFull
        } else {
            CellState::Full
        }
    }
    pub fn max_per_type(&self) -> u64 {
        if !self.equal_distribution {
            return u64::MAX;
        }
        let max_types = if self.fuzzy.is_none() && !self.inverted && !self.partition.is_empty() {
            self.partition.len().min(self.layout.total_types)
        } else {
            self.layout.total_types
        } as u64;
        let total = (self.layout.bytes - self.layout.bytes_per_type * max_types)
            * self.kind.amount_per_byte();
        total.div_ceil(max_types)
    }

    /// Returns the accepted amount, including voided overflow, as MEStorage does.
    /// Simulation must return the same answer without mutating the inventory.
    pub fn insert(&mut self, key: K, amount: u64, commit: bool) -> u64 {
        if amount == 0 || key.key_type() != self.kind || key.blacklisted() {
            return 0;
        }
        let listed = match self.fuzzy {
            Some(mode) => self
                .partition
                .iter()
                .any(|filter| filter.fuzzy_search_matches(&key, mode)),
            None => self.partition.contains(&key),
        };
        if !self.partition.is_empty() && listed == self.inverted {
            return 0;
        }
        // Empty inverted partitions permit everything, like IPartitionList.
        let inserted = self.inner_insert(key.clone(), amount, commit);
        if self.partition.is_empty() && self.void && !self.can_hold_new_type() {
            if self.stored.contains_key(&key) {
                amount
            } else {
                inserted
            }
        } else if self.void {
            amount
        } else {
            inserted
        }
    }

    fn inner_insert(&mut self, key: K, amount: u64, commit: bool) -> u64 {
        if !key.can_fit_inside_cell() {
            return 0;
        }
        let current = self.stored.get(&key).copied().unwrap_or(0);
        let mut remaining = self.remaining_count();
        if current == 0 {
            if !self.can_hold_new_type() {
                return 0;
            }
            remaining =
                remaining.saturating_sub(self.layout.bytes_per_type * self.kind.amount_per_byte());
            if remaining == 0 {
                return 0;
            }
        }
        let accepted = amount.min(remaining.min(self.max_per_type().saturating_sub(current)));
        if commit {
            self.stored.insert(key, current + accepted);
        }
        accepted
    }

    /// Extraction deliberately ignores the current partition; old items remain removable.
    pub fn extract(&mut self, key: &K, amount: u64, commit: bool) -> u64 {
        let current = self.stored.get(key).copied().unwrap_or(0);
        let extracted = amount.min(current);
        if commit && current > 0 {
            if extracted == current {
                self.stored.remove(key);
            } else {
                self.stored.insert(key.clone(), current - extracted);
            }
        }
        extracted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Key(u32, u32); // Registry identity and component identity.
    impl CellKey for Key {
        fn key_type(&self) -> KeyType {
            if self.0 == 999 {
                KeyType::Fluid
            } else {
                KeyType::Item
            }
        }
        fn can_fit_inside_cell(&self) -> bool {
            self.0 != 998
        }
        fn blacklisted(&self) -> bool {
            self.0 == 997
        }
    }

    fn cell() -> Cell<Key> {
        Cell::new(Tier::K1, KeyType::Item)
    }

    #[test]
    fn portable_layout_uses_half_bytes_original_overhead_and_decreasing_item_types() {
        for (tier, types) in Tier::ALL.into_iter().zip([54, 45, 36, 27, 18]) {
            let layout = CellLayout::portable(tier, KeyType::Item);
            assert_eq!(layout.bytes(), tier.bytes() / 2);
            assert_eq!(layout.bytes_per_type(), tier.bytes() / 128);
            assert_eq!(layout.total_types(), types);
            assert_eq!(layout.idle_drain(), 0.5);
            let mut cell = Cell::new_with_layout(tier, KeyType::Item, layout);
            for id in 0..types as u32 {
                assert_eq!(cell.insert(Key(id, 0), 1, true), 1);
            }
            assert_eq!(cell.insert(Key(100, 0), 1, true), 0);
            assert_eq!(cell.state(), CellState::TypesFull);
            let fluid = CellLayout::portable(tier, KeyType::Fluid);
            assert_eq!(fluid.total_types(), 18);
            let mut fluid = Cell::new_with_layout(tier, KeyType::Fluid, fluid);
            assert_eq!(
                fluid.insert(Key(999, 0), u64::MAX, true),
                (layout.bytes() - layout.bytes_per_type()) * 8000
            );
            assert_eq!(fluid.state(), CellState::Full);
        }
    }

    #[test]
    fn portable_restore_enforces_its_own_capacity_and_type_limit() {
        let layout = CellLayout::portable(Tier::K1, KeyType::Item);
        assert!(
            Cell::restore_with_layout(
                Tier::K1,
                KeyType::Item,
                layout,
                BTreeMap::from([(Key(1, 0), 4032)])
            )
            .is_ok()
        );
        assert!(
            Cell::restore_with_layout(
                Tier::K1,
                KeyType::Item,
                layout,
                BTreeMap::from([(Key(1, 0), 4033)])
            )
            .is_err()
        );
        let layout = CellLayout::portable(Tier::K256, KeyType::Item);
        let stacks = (0..19).map(|id| (Key(id, 0), 1)).collect();
        assert!(Cell::restore_with_layout(Tier::K256, KeyType::Item, layout, stacks).is_err());
    }

    #[test]
    fn restore_does_not_reinsert_through_filters_and_rejects_overcapacity() {
        let contents = BTreeMap::from([(Key(1, 0), 8000)]);
        let mut c = Cell::restore(Tier::K1, KeyType::Item, contents).unwrap();
        c.configure(BTreeSet::from([Key(2, 0)]), false, false, false);
        assert_eq!(c.extract(&Key(1, 0), 8000, true), 8000);
        assert!(
            Cell::restore(Tier::K1, KeyType::Item, BTreeMap::from([(Key(1, 0), 9000)])).is_err()
        );
        assert!(
            Cell::restore(Tier::K1, KeyType::Item, BTreeMap::from([(Key(999, 0), 1)])).is_err()
        );
    }

    #[test]
    fn fuzzy_cell_partitions_and_distribution_follow_variant_search() {
        use super::super::ae2_keys::{Key as ResourceKey, KeyProperties};
        let key = |damage| {
            ResourceKey::new(
                KeyType::Item,
                "minecraft:diamond_pickaxe".into(),
                vec![damage],
                KeyProperties {
                    damage: i32::from(damage),
                    max_damage: 100,
                    ..Default::default()
                },
            )
            .unwrap()
        };
        let mut c = Cell::new(Tier::K1, KeyType::Item);
        c.configure(BTreeSet::from([key(25)]), false, true, false);
        assert_eq!(c.max_per_type(), 8128);
        c.set_fuzzy(Some(FuzzyMode::Percent75));
        assert_eq!(c.max_per_type(), 67); // ceil((1024-63*8)*8/63), not whitelist size
        assert_eq!(c.insert(key(26), 1, true), 0);
        assert_eq!(c.insert(key(0), 100, true), 67);
        c.configure(BTreeSet::from([key(25)]), true, false, false);
        assert_eq!(c.insert(key(1), 1, true), 0);
        assert_eq!(c.insert(key(26), 1, true), 1);
    }

    #[test]
    fn all_tiers_single_type_capacity_and_simulation() {
        for tier in Tier::ALL {
            let mut c = Cell::new(tier, KeyType::Item);
            let expected = (tier.bytes() - tier.bytes_per_type()) * 8;
            assert_eq!(c.insert(Key(1, 0), u64::MAX, false), expected);
            assert_eq!(c.state(), CellState::Empty);
            assert_eq!(c.insert(Key(1, 0), u64::MAX, true), expected);
            assert_eq!(c.state(), CellState::Full);
            assert_eq!(c.used_bytes(), tier.bytes());
            assert_eq!(c.extract(&Key(1, 0), u64::MAX, false), expected);
            assert_eq!(c.stored_count(), expected);
            c.extract(&Key(1, 0), u64::MAX, true);
            assert_eq!(c.used_bytes(), 0);
        }
    }

    #[test]
    fn sixty_three_types_and_components_are_distinct() {
        let mut c = cell();
        for component in 0..63 {
            assert_eq!(c.insert(Key(1, component), 1, true), 1);
        }
        assert_eq!(c.state(), CellState::TypesFull);
        assert_eq!(c.insert(Key(2, 0), 1, true), 0);
        assert_eq!(c.insert(Key(1, 0), 99, true), 99);
        assert_eq!(c.used_bytes(), 63 * 8 + 162u64.div_ceil(8));
    }

    #[test]
    fn shared_rounding_credit_can_pay_for_first_item_of_new_type() {
        let mut c = cell();
        assert_eq!(c.insert(Key(1, 0), 8057, true), 8057);
        assert_eq!(c.free_bytes(), 8);
        assert_eq!(c.unused_count(), 7);
        assert!(c.can_hold_new_type());
        assert_eq!(c.insert(Key(2, 0), 8, true), 7);
        assert_eq!(c.state(), CellState::Full);
    }

    #[test]
    fn fluid_cells_use_millibuckets_and_eighteen_types() {
        let mut c = Cell::new(Tier::K1, KeyType::Fluid);
        assert_eq!(c.insert(Key(1, 0), 1, true), 0);
        assert_eq!(c.insert(Key(999, 0), u64::MAX, true), 8_128_000);
        assert_eq!(c.state(), CellState::Full);
        assert_eq!(KeyType::Fluid.total_types(), 18);
    }

    #[test]
    fn exact_partition_inversion_and_removal_after_filter_change() {
        let mut c = cell();
        c.configure(BTreeSet::from([Key(1, 0)]), false, false, false);
        assert_eq!(c.insert(Key(2, 0), 1, true), 0);
        assert_eq!(c.insert(Key(1, 0), 1, true), 1);
        c.configure(BTreeSet::from([Key(1, 0)]), true, false, false);
        assert_eq!(c.insert(Key(1, 0), 1, true), 0);
        assert_eq!(c.extract(&Key(1, 0), 1, true), 1);
        assert_eq!(c.insert(Key(2, 0), 1, true), 1);
    }

    #[test]
    fn equal_distribution_uses_whitelist_and_rounds_up() {
        let mut c = cell();
        c.configure(
            BTreeSet::from([Key(1, 0), Key(2, 0), Key(3, 0)]),
            false,
            true,
            false,
        );
        assert_eq!(c.max_per_type(), 2667);
        assert_eq!(c.insert(Key(1, 0), u64::MAX, true), 2667);
        assert_eq!(c.insert(Key(1, 0), 1, true), 0);
    }

    #[test]
    fn void_card_on_unformatted_full_cell_cannot_void_new_types() {
        let mut c = cell();
        c.configure(BTreeSet::new(), false, false, true);
        assert_eq!(c.insert(Key(1, 0), 9000, true), 9000);
        assert_eq!(c.stored_count(), 8128);
        assert_eq!(c.insert(Key(2, 0), 10, true), 0);
        assert_eq!(c.insert(Key(1, 0), 10, true), 10);
    }

    #[test]
    fn blacklist_and_nested_cell_rejected_without_void() {
        let mut c = cell();
        for key in [Key(997, 0), Key(998, 0), Key(999, 0)] {
            assert_eq!(c.insert(key, 1, true), 0);
        }
        assert_eq!(c.state(), CellState::Empty);
    }

    #[test]
    fn deterministic_mixed_operations_conserve_items_and_capacity() {
        let mut c = cell();
        let mut state = 42u64;
        let mut balance = 0;
        for _ in 0..10000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let key = Key((state >> 32) as u32 % 80, 0);
            let amount = (state >> 40) % 256;
            if state & 1 == 0 {
                balance += c.insert(key, amount, true);
            } else {
                balance -= c.extract(&key, amount, true);
            }
            assert_eq!(c.stored_count(), balance);
            assert!(c.used_bytes() <= 1024);
            assert!(c.stored_types() <= 63);
        }
    }
}
