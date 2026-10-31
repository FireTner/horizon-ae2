// SPDX-License-Identifier: LGPL-3.0-or-later
// NetworkStorage mount/order/recursion rules from pinned AE2.
// Copyright AlgorithmX2 / TeamAppliedEnergistics; 2026 Horizon port contributors.
// This layer forwards an opaque action source to providers. Authorization,
// energy costs, provider lifecycle, world hosts and the terminal live elsewhere.

use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

/// Providers may add signed counts (including crafting estimates). Preserve
/// Java long arithmetic when combining them, as KeyCounter does.
pub type Counts<K> = BTreeMap<K, i64>;
pub fn add_count<K: Ord>(out: &mut Counts<K>, key: K, amount: i64) {
    let entry = out.entry(key).or_default();
    *entry = entry.wrapping_add(amount);
}

/// Native equivalent of the MEStorage operations. `S` is an opaque action
/// source forwarded unchanged, so this layer cannot invent a permission grant.
pub trait Storage<K: Ord, S>: Send {
    fn preferred(&mut self, _key: &K, _source: &S) -> bool {
        false
    }
    fn insert(&mut self, _key: &K, _amount: u64, _commit: bool, _source: &S) -> u64 {
        0
    }
    fn extract(&mut self, _key: &K, _amount: u64, _commit: bool, _source: &S) -> u64 {
        0
    }
    fn available(&mut self, _out: &mut Counts<K>) {}
}

pub type Inventory<K, S> = Arc<Mutex<dyn Storage<K, S>>>;
type Groups<K, S> = BTreeMap<Reverse<i32>, Vec<Inventory<K, S>>>;
enum Queued<K: Ord, S> {
    Mount(i32, Inventory<K, S>),
    Unmount(Inventory<K, S>),
}
struct MountTable<K: Ord, S> {
    busy: bool,
    groups: Groups<K, S>,
    queued: Vec<Queued<K, S>>,
}

/// Shared identity is required for subnetworks and cyclic storage-bus graphs.
/// A recursive call observes `busy` before locking any provider, and returns 0.
pub struct NetworkStorage<K: Ord, S> {
    table: Arc<Mutex<MountTable<K, S>>>,
}
/// Use weak handles for mounted subnetwork links and provider callbacks. The
/// grid service owns the strong handle; cyclic buses must not keep dead grids alive.
pub struct WeakNetworkStorage<K: Ord, S> {
    table: Weak<Mutex<MountTable<K, S>>>,
}
impl<K: Ord, S> Clone for WeakNetworkStorage<K, S> {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
        }
    }
}
impl<K: Ord, S> WeakNetworkStorage<K, S> {
    pub fn upgrade(&self) -> Option<NetworkStorage<K, S>> {
        self.table.upgrade().map(|table| NetworkStorage { table })
    }
}
impl<K: Ord, S> Clone for NetworkStorage<K, S> {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
        }
    }
}
impl<K: Ord, S> Default for NetworkStorage<K, S> {
    fn default() -> Self {
        Self {
            table: Arc::new(Mutex::new(MountTable {
                busy: false,
                groups: BTreeMap::new(),
                queued: vec![],
            })),
        }
    }
}

struct Operation<'a, K: Ord, S> {
    network: &'a NetworkStorage<K, S>,
    groups: Groups<K, S>,
}
impl<K: Ord, S> Drop for Operation<'_, K, S> {
    fn drop(&mut self) {
        self.network.table.lock().unwrap().busy = false;
    }
}

impl<K: Ord, S> NetworkStorage<K, S> {
    pub fn weak(&self) -> WeakNetworkStorage<K, S> {
        WeakNetworkStorage {
            table: Arc::downgrade(&self.table),
        }
    }
    pub fn mount(&self, priority: i32, inventory: Inventory<K, S>) {
        let mut table = self.table.lock().unwrap();
        if table.busy {
            table.queued.push(Queued::Mount(priority, inventory));
        } else {
            table
                .groups
                .entry(Reverse(priority))
                .or_default()
                .push(inventory);
        }
    }
    pub fn unmount(&self, inventory: &Inventory<K, S>) {
        let mut table = self.table.lock().unwrap();
        if table.busy {
            table.queued.push(Queued::Unmount(inventory.clone()));
        } else {
            // Java List.remove removes one matching occurrence per priority,
            // even when the same inventory was mounted more than once.
            table.groups.retain(|_, group| {
                if let Some(index) = group.iter().position(|item| Arc::ptr_eq(item, inventory)) {
                    group.remove(index);
                }
                !group.is_empty()
            });
        }
    }
    fn begin(&self) -> Option<Operation<'_, K, S>> {
        let mut table = self.table.lock().unwrap();
        if table.busy {
            return None;
        }
        table.busy = true;
        Some(Operation {
            network: self,
            groups: table.groups.clone(),
        })
    }
    fn queued_for_removal(&self, inventory: &Inventory<K, S>) -> bool {
        self.table
            .lock()
            .unwrap()
            .queued
            .iter()
            .any(|op| matches!(op, Queued::Unmount(other) if Arc::ptr_eq(other, inventory)))
    }
    fn flush(&self) {
        let queued = {
            let mut table = self.table.lock().unwrap();
            assert!(!table.busy);
            std::mem::take(&mut table.queued)
        };
        for op in queued {
            match op {
                Queued::Mount(priority, inventory) => self.mount(priority, inventory),
                Queued::Unmount(inventory) => self.unmount(&inventory),
            }
        }
    }
    pub fn insert(&self, key: &K, amount: u64, commit: bool, source: &S) -> u64 {
        assert!(amount <= i64::MAX as u64);
        let Some(op) = self.begin() else { return 0 };
        let mut remaining = amount;
        for group in op.groups.values() {
            let mut second_pass = vec![];
            for inventory in group {
                if remaining == 0 {
                    break;
                }
                if self.queued_for_removal(inventory) {
                    continue;
                }
                if inventory.lock().unwrap().preferred(key, source) {
                    let accepted = inventory
                        .lock()
                        .unwrap()
                        .insert(key, remaining, commit, source);
                    assert!(accepted <= remaining, "storage returned excess insertion");
                    remaining -= accepted;
                } else {
                    second_pass.push(inventory);
                }
            }
            for inventory in second_pass {
                if remaining == 0 {
                    break;
                }
                if self.queued_for_removal(inventory) {
                    continue;
                }
                let accepted = inventory
                    .lock()
                    .unwrap()
                    .insert(key, remaining, commit, source);
                assert!(accepted <= remaining, "storage returned excess insertion");
                remaining -= accepted;
            }
        }
        drop(op);
        self.flush();
        amount - remaining
    }
    pub fn extract(&self, key: &K, amount: u64, commit: bool, source: &S) -> u64 {
        assert!(amount <= i64::MAX as u64);
        let Some(op) = self.begin() else { return 0 };
        let mut extracted = 0;
        for group in op.groups.values().rev() {
            for inventory in group {
                if extracted >= amount {
                    break;
                }
                if self.queued_for_removal(inventory) {
                    continue;
                }
                let value =
                    inventory
                        .lock()
                        .unwrap()
                        .extract(key, amount - extracted, commit, source);
                assert!(
                    value <= amount - extracted,
                    "storage returned excess extraction"
                );
                extracted += value;
            }
        }
        drop(op);
        self.flush();
        extracted
    }
    pub fn available(&self, out: &mut Counts<K>) {
        let Some(op) = self.begin() else { return };
        for group in op.groups.values() {
            for inventory in group {
                inventory.lock().unwrap().available(out);
            }
        }
        // Original getAvailableStacks deliberately does not flush operations
        // queued by providers during listing, nor skip queued removals.
    }
}
impl<K: Ord + Send, S> Storage<K, S> for NetworkStorage<K, S> {
    fn insert(&mut self, key: &K, amount: u64, commit: bool, source: &S) -> u64 {
        NetworkStorage::insert(self, key, amount, commit, source)
    }
    fn extract(&mut self, key: &K, amount: u64, commit: bool, source: &S) -> u64 {
        NetworkStorage::extract(self, key, amount, commit, source)
    }
    fn available(&mut self, out: &mut Counts<K>) {
        NetworkStorage::available(self, out);
    }
}
impl<K: Ord + Send, S> Storage<K, S> for WeakNetworkStorage<K, S> {
    fn insert(&mut self, key: &K, amount: u64, commit: bool, source: &S) -> u64 {
        self.upgrade()
            .map_or(0, |net| net.insert(key, amount, commit, source))
    }
    fn extract(&mut self, key: &K, amount: u64, commit: bool, source: &S) -> u64 {
        self.upgrade()
            .map_or(0, |net| net.extract(key, amount, commit, source))
    }
    fn available(&mut self, out: &mut Counts<K>) {
        if let Some(net) = self.upgrade() {
            net.available(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Bin {
        count: u64,
        capacity: u64,
        preferred: bool,
        seen_source: u64,
    }
    impl Storage<u32, u64> for Bin {
        fn preferred(&mut self, _: &u32, source: &u64) -> bool {
            self.seen_source = *source;
            self.preferred
        }
        fn insert(&mut self, _: &u32, amount: u64, commit: bool, source: &u64) -> u64 {
            self.seen_source = *source;
            let accepted = amount.min(self.capacity - self.count);
            if commit {
                self.count += accepted;
            }
            accepted
        }
        fn extract(&mut self, _: &u32, amount: u64, commit: bool, source: &u64) -> u64 {
            self.seen_source = *source;
            let extracted = amount.min(self.count);
            if commit {
                self.count -= extracted;
            }
            extracted
        }
        fn available(&mut self, out: &mut Counts<u32>) {
            add_count(out, 1, self.count as i64);
        }
    }
    fn bin(preferred: bool) -> Arc<Mutex<Bin>> {
        Arc::new(Mutex::new(Bin {
            capacity: 100,
            preferred,
            ..Bin::default()
        }))
    }
    #[test]
    fn insertion_is_high_priority_then_preferred_within_each_priority() {
        let net = NetworkStorage::default();
        let high = bin(false);
        let low = bin(true);
        let ordinary = bin(false);
        net.mount(0, ordinary.clone());
        net.mount(0, low.clone());
        net.mount(10, high.clone());
        assert_eq!(net.insert(&1, 120, false, &77), 120);
        assert_eq!(high.lock().unwrap().count, 0);
        assert_eq!(net.insert(&1, 120, true, &77), 120);
        assert_eq!(high.lock().unwrap().count, 100);
        assert_eq!(low.lock().unwrap().count, 20);
        assert_eq!(ordinary.lock().unwrap().count, 0);
        assert_eq!(low.lock().unwrap().seen_source, 77);
        assert_eq!(net.extract(&1, 25, true, &88), 25);
        assert_eq!(low.lock().unwrap().count, 0);
        assert_eq!(high.lock().unwrap().count, 95);
        assert_eq!(high.lock().unwrap().seen_source, 88);
    }
    #[test]
    fn unmount_removes_only_one_occurrence_from_each_priority() {
        let net = NetworkStorage::default();
        let a: Inventory<_, _> = bin(false);
        net.mount(0, a.clone());
        net.mount(0, a.clone());
        net.mount(10, a.clone());
        net.unmount(&a);
        assert_eq!(net.insert(&1, 100, true, &0), 100);
        let mut counts = Counts::new();
        net.available(&mut counts);
        assert_eq!(counts[&1], 100);
        net.unmount(&a);
        let mut counts = Counts::new();
        net.available(&mut counts);
        assert!(counts.is_empty());
    }
    #[test]
    fn cyclic_subnetworks_return_zero_before_locking_active_provider() {
        let a = NetworkStorage::<u32, u64>::default();
        let b = NetworkStorage::default();
        a.mount(0, Arc::new(Mutex::new(b.weak())));
        b.mount(0, Arc::new(Mutex::new(a.weak())));
        let leaf = bin(false);
        b.mount(-1, leaf.clone());
        assert_eq!(a.insert(&1, 10, true, &0), 10);
        assert_eq!(a.extract(&1, 10, true, &0), 10);
        a.insert(&1, 7, true, &0);
        let mut out = Counts::new();
        a.available(&mut out);
        assert_eq!(out[&1], 7);
        let weak_a = a.weak();
        let weak_b = b.weak();
        drop(a);
        drop(b);
        assert!(weak_a.upgrade().is_none());
        assert!(weak_b.upgrade().is_none());
    }
    struct Callback {
        net: NetworkStorage<u32, u64>,
        target: Inventory<u32, u64>,
        on_list: bool,
    }
    impl Storage<u32, u64> for Callback {
        fn insert(&mut self, _: &u32, _: u64, _: bool, _: &u64) -> u64 {
            self.net.unmount(&self.target);
            self.net.mount(-5, self.target.clone());
            0
        }
        fn available(&mut self, _: &mut Counts<u32>) {
            if self.on_list {
                self.net.unmount(&self.target);
            }
        }
    }
    #[test]
    fn queued_removal_skips_provider_and_later_remount_waits_until_next_operation() {
        let net = NetworkStorage::default();
        let leaf = bin(true);
        let callback: Inventory<_, _> = Arc::new(Mutex::new(Callback {
            net: net.clone(),
            target: leaf.clone(),
            on_list: false,
        }));
        net.mount(10, callback.clone());
        net.mount(0, leaf.clone());
        assert_eq!(net.insert(&1, 10, true, &0), 0);
        net.unmount(&callback);
        assert_eq!(net.insert(&1, 10, true, &0), 10);
        assert_eq!(leaf.lock().unwrap().count, 10);
    }
    #[test]
    fn listing_keeps_queued_removals_until_next_insert_even_when_amount_is_zero() {
        let net = NetworkStorage::default();
        let leaf = bin(true);
        leaf.lock().unwrap().count = 42;
        let callback: Inventory<_, _> = Arc::new(Mutex::new(Callback {
            net: net.clone(),
            target: leaf.clone(),
            on_list: true,
        }));
        net.mount(10, callback.clone());
        net.mount(0, leaf.clone());
        let mut out = Counts::new();
        net.available(&mut out);
        assert_eq!(out[&1], 42);
        let mut out = Counts::new();
        net.available(&mut out);
        assert_eq!(out[&1], 42);
        net.insert(&1, 0, true, &0);
        let mut out = Counts::new();
        net.available(&mut out);
        assert!(out.is_empty());
        net.unmount(&callback);
    }
    #[test]
    fn counters_keep_signed_java_long_addition() {
        let mut counts = Counts::new();
        add_count(&mut counts, 1, i64::MAX);
        add_count(&mut counts, 1, 1);
        assert_eq!(counts[&1], i64::MIN);
    }
}
