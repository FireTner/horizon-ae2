// SPDX-License-Identifier: LGPL-3.0-or-later
// StorageCell backends: BasicCellInventory and CreativeCellInventory.
// Copyright TeamAppliedEnergistics; 2026 Horizon port contributors.
use super::ae2_storage::{Cell, CellKey, CellState};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct CreativeCell<K> {
    configured: BTreeMap<K, u64>,
}
impl<K: Ord> CreativeCell<K> {
    pub fn new(keys: BTreeSet<K>) -> Self {
        Self {
            configured: keys.into_iter().map(|key| (key, i32::MAX as u64)).collect(),
        }
    }
    pub fn stacks(&self) -> &BTreeMap<K, u64> {
        &self.configured
    }
    pub fn insert(&self, key: &K, amount: u64, _commit: bool) -> u64 {
        if self.configured.contains_key(key) {
            amount
        } else {
            0
        }
    }
    pub fn extract(&self, key: &K, amount: u64, _commit: bool) -> u64 {
        if self.configured.contains_key(key) {
            amount
        } else {
            0
        }
    }
    pub fn preferred(&self, key: &K) -> bool {
        self.configured.contains_key(key)
    }
    pub fn can_fit_inside_cell(&self) -> bool {
        self.configured.is_empty()
    }
    pub fn state(&self) -> CellState {
        CellState::TypesFull
    }
    pub fn idle_drain(&self) -> f64 {
        0.0
    }
}

#[derive(Clone, Debug)]
pub enum CellBackend<K> {
    Basic(Cell<K>),
    Creative(CreativeCell<K>),
    Restricted {
        cell: Cell<K>,
        allowed: BTreeSet<String>,
    },
}
impl<K> From<Cell<K>> for CellBackend<K> {
    fn from(cell: Cell<K>) -> Self {
        Self::Basic(cell)
    }
}
impl<K> From<CreativeCell<K>> for CellBackend<K> {
    fn from(cell: CreativeCell<K>) -> Self {
        Self::Creative(cell)
    }
}
impl<K: CellKey> CellBackend<K> {
    pub fn finite(&self) -> Option<&Cell<K>> {
        match self {
            Self::Basic(cell) | Self::Restricted { cell, .. } => Some(cell),
            Self::Creative(_) => None,
        }
    }
    pub fn restricted(cell: Cell<K>, allowed: BTreeSet<String>) -> Self {
        Self::Restricted { cell, allowed }
    }
    pub fn state(&self) -> CellState {
        match self {
            Self::Basic(c) => c.state(),
            Self::Creative(c) => c.state(),
            Self::Restricted { cell, .. } => cell.state(),
        }
    }
    pub fn idle_drain(&self) -> f64 {
        match self {
            Self::Basic(c) => c.layout().idle_drain(),
            Self::Creative(c) => c.idle_drain(),
            Self::Restricted { cell, .. } => cell.layout().idle_drain(),
        }
    }
    pub fn insert(&mut self, key: K, amount: u64, commit: bool) -> u64 {
        match self {
            Self::Basic(c) => c.insert(key, amount, commit),
            Self::Creative(c) => c.insert(&key, amount, commit),
            Self::Restricted { cell, allowed } => {
                if key.primary_id().is_some_and(|id| allowed.contains(id)) {
                    cell.insert(key, amount, commit)
                } else {
                    0
                }
            }
        }
    }
    pub fn extract(&mut self, key: &K, amount: u64, commit: bool) -> u64 {
        match self {
            Self::Basic(c) => c.extract(key, amount, commit),
            Self::Creative(c) => c.extract(key, amount, commit),
            Self::Restricted { cell, .. } => cell.extract(key, amount, commit),
        }
    }
    pub fn stacks(&self) -> &BTreeMap<K, u64> {
        match self {
            Self::Basic(c) => c.stacks(),
            Self::Creative(c) => c.stacks(),
            Self::Restricted { cell, .. } => cell.stacks(),
        }
    }
    /// Sum of listed amounts; creative listings are capped at Integer.MAX_VALUE
    /// per configured key, even though each extraction is unbounded.
    pub fn stored_count(&self) -> u64 {
        self.stacks().values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        ae2_keys::{Key, KeyType},
        ae2_storage::{CellLayout, Tier},
    };
    use super::*;
    fn item(id: &str, variant: u8) -> Key {
        Key::new(
            KeyType::Item,
            id.to_owned(),
            vec![variant],
            Default::default(),
        )
        .unwrap()
    }
    #[test]
    fn tool_layouts_apply_exact_capacity_and_primary_item_admission_without_losing_variants() {
        for (tier, layout, capacity) in [
            (Tier::K1, CellLayout::matter_cannon(), 4032),
            (Tier::K4, CellLayout::color_applicator(), 16128),
        ] {
            let mut backend = CellBackend::restricted(
                Cell::new_with_layout(tier, KeyType::Item, layout),
                BTreeSet::from(["ae2:red_paint_ball".to_owned()]),
            );
            assert_eq!(backend.insert(item("minecraft:stone", 0), 1, true), 0);
            assert_eq!(
                backend.insert(item("ae2:red_paint_ball", 1), u64::MAX, false),
                capacity
            );
            assert_eq!(backend.stored_count(), 0);
            assert_eq!(
                backend.insert(item("ae2:red_paint_ball", 1), u64::MAX, true),
                capacity
            );
            assert_eq!(backend.extract(&item("ae2:red_paint_ball", 0), 10, true), 0);
            assert_eq!(
                backend.extract(&item("ae2:red_paint_ball", 1), u64::MAX, true),
                capacity
            );
            assert_eq!(backend.idle_drain(), 0.5);
        }
        let mut backend = CellBackend::restricted(
            Cell::new_with_layout(Tier::K4, KeyType::Item, CellLayout::color_applicator()),
            BTreeSet::from(["minecraft:red_dye".to_owned()]),
        );
        for variant in 0..27 {
            assert_eq!(
                backend.insert(item("minecraft:red_dye", variant), 1, true),
                1
            );
        }
        assert_eq!(backend.insert(item("minecraft:red_dye", 27), 1, true), 0);
        assert_eq!(backend.state(), CellState::TypesFull);
    }
    #[test]
    fn tool_blacklist_precedes_void_and_old_disallowed_contents_remain_extractable() {
        let stone = item("minecraft:stone", 0);
        let mut cell =
            Cell::new_with_layout(Tier::K4, KeyType::Item, CellLayout::color_applicator());
        assert_eq!(cell.insert(stone.clone(), 100, true), 100);
        cell.configure(BTreeSet::new(), false, false, true);
        let mut backend =
            CellBackend::restricted(cell, BTreeSet::from(["minecraft:red_dye".to_owned()]));
        assert_eq!(backend.insert(stone.clone(), 100, true), 0);
        assert_eq!(backend.extract(&stone, 100, true), 100);
        assert_eq!(
            backend.insert(item("minecraft:red_dye", 0), 100_000, true),
            100_000
        );
        assert_eq!(backend.stored_count(), 16128);
    }
    #[test]
    fn creative_insert_extract_never_mutate_and_preserve_exact_configured_keys() {
        let cell = CreativeCell::new(BTreeSet::from([("iron", 0), ("tool", 17)]));
        let original = cell.stacks().clone();
        for commit in [false, true] {
            for amount in [0, 1, i32::MAX as u64 + 1, i64::MAX as u64] {
                assert_eq!(cell.insert(&("iron", 0), amount, commit), amount);
                assert_eq!(cell.extract(&("iron", 0), amount, commit), amount);
                assert_eq!(cell.insert(&("tool", 18), amount, commit), 0);
                assert_eq!(cell.extract(&("tool", 18), amount, commit), 0);
                assert_eq!(cell.stacks(), &original);
            }
        }
        assert_eq!(cell.stacks()[&("iron", 0)], i32::MAX as u64);
        assert!(cell.preferred(&("tool", 17)));
        assert!(!cell.preferred(&("tool", 18)));
        assert!(!cell.can_fit_inside_cell());
        assert_eq!(cell.state(), CellState::TypesFull);
        assert_eq!(cell.idle_drain(), 0.0);
    }
    #[test]
    fn empty_creative_cell_keeps_types_full_status_zero_drain_and_can_be_nested() {
        let cell = CreativeCell::<u32>::new(BTreeSet::new());
        assert_eq!(cell.state(), CellState::TypesFull);
        assert_eq!(cell.idle_drain(), 0.0);
        assert!(cell.can_fit_inside_cell());
        assert_eq!(cell.extract(&1, 10, true), 0);
        assert_eq!(cell.insert(&1, 10, true), 0);
        assert!(cell.stacks().is_empty());
    }
}
