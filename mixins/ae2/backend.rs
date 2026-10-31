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
    pub fn state(&self) -> CellState {
        match self {
            Self::Basic(c) => c.state(),
            Self::Creative(c) => c.state(),
        }
    }
    pub fn idle_drain(&self) -> f64 {
        match self {
            Self::Basic(c) => c.layout().idle_drain(),
            Self::Creative(c) => c.idle_drain(),
        }
    }
    pub fn insert(&mut self, key: K, amount: u64, commit: bool) -> u64 {
        match self {
            Self::Basic(c) => c.insert(key, amount, commit),
            Self::Creative(c) => c.insert(&key, amount, commit),
        }
    }
    pub fn extract(&mut self, key: &K, amount: u64, commit: bool) -> u64 {
        match self {
            Self::Basic(c) => c.extract(key, amount, commit),
            Self::Creative(c) => c.extract(key, amount, commit),
        }
    }
    pub fn stacks(&self) -> &BTreeMap<K, u64> {
        match self {
            Self::Basic(c) => c.stacks(),
            Self::Creative(c) => c.stacks(),
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
    use super::*;
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
