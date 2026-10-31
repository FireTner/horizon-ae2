// SPDX-License-Identifier: LGPL-3.0-or-later
// StorageService / ProviderState / InterestManager / StackWatcher rules from AE2.
// Copyright AlgorithmX2 / TeamAppliedEnergistics; 2026 Horizon port contributors.
// The caller delivers notifications synchronously. Host identity uses stable IDs;
// Java hash iteration order and faults from callbacks mutating live iterators are
// not a portable ordering contract. World registration lives in the adapter.

use super::ae2_mounts::{Counts, Inventory, NetworkStorage};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
pub type NodeId = u64;
pub type WatcherId = u64;

pub trait StorageProvider<K: Ord, S>: Send {
    fn mount(&mut self, mounts: &mut StorageMounts<'_, K, S>) -> Result<(), &'static str>;
}
pub type Provider<K, S> = Arc<Mutex<dyn StorageProvider<K, S>>>;

pub struct StorageMounts<'a, K: Ord, S> {
    network: &'a NetworkStorage<K, S>,
    inventories: &'a mut Vec<Inventory<K, S>>,
}
impl<K: Ord, S> StorageMounts<'_, K, S> {
    pub fn mount(&mut self, inventory: Inventory<K, S>, priority: i32) -> Result<(), &'static str> {
        if self.inventories.iter().any(|i| Arc::ptr_eq(i, &inventory)) {
            return Err("cannot mount the same inventory twice");
        }
        self.inventories.push(inventory.clone());
        self.network.mount(priority, inventory);
        Ok(())
    }
}
struct ProviderState<K: Ord, S> {
    provider: Provider<K, S>,
    inventories: Vec<Inventory<K, S>>,
    mounted: bool,
}
impl<K: Ord, S> ProviderState<K, S> {
    fn new(provider: Provider<K, S>) -> Self {
        Self {
            provider,
            inventories: vec![],
            mounted: false,
        }
    }
    fn mount(&mut self, network: &NetworkStorage<K, S>) -> Result<(), &'static str> {
        if self.mounted {
            return Err("provider already mounted");
        }
        self.mounted = true;
        self.provider.lock().unwrap().mount(&mut StorageMounts {
            network,
            inventories: &mut self.inventories,
        })
    }
    fn unmount(&mut self, network: &NetworkStorage<K, S>) {
        if !self.mounted {
            return;
        }
        self.mounted = false;
        for inventory in &self.inventories {
            network.unmount(inventory);
        }
        self.inventories.clear();
    }
    fn update(&mut self, network: &NetworkStorage<K, S>) -> Result<(), &'static str> {
        self.unmount(network);
        self.mount(network)
    }
}

struct Interests<K> {
    keys: BTreeSet<K>,
    all: bool,
    destroyed: bool,
}
pub struct StackWatcher<K> {
    interests: Arc<Mutex<Interests<K>>>,
}
impl<K> Clone for StackWatcher<K> {
    fn clone(&self) -> Self {
        Self {
            interests: self.interests.clone(),
        }
    }
}
impl<K: Ord> Default for StackWatcher<K> {
    fn default() -> Self {
        Self {
            interests: Arc::new(Mutex::new(Interests {
                keys: BTreeSet::new(),
                all: false,
                destroyed: false,
            })),
        }
    }
}
impl<K: Ord> StackWatcher<K> {
    pub fn add(&self, key: K) {
        let mut state = self.interests.lock().unwrap();
        if !state.destroyed {
            state.keys.insert(key);
        }
    }
    pub fn remove(&self, key: &K) {
        let mut state = self.interests.lock().unwrap();
        if !state.destroyed {
            state.keys.remove(key);
        }
    }
    pub fn watch_all(&self, all: bool) {
        let mut state = self.interests.lock().unwrap();
        if !state.destroyed {
            state.all = all;
        }
    }
    pub fn reset(&self) {
        let mut state = self.interests.lock().unwrap();
        state.all = false;
        state.keys.clear();
    }
    pub fn destroy(&self) {
        let mut state = self.interests.lock().unwrap();
        state.all = false;
        state.keys.clear();
        state.destroyed = true;
    }
    fn empty(&self) -> bool {
        let state = self.interests.lock().unwrap();
        !state.all && state.keys.is_empty()
    }
}

pub struct StorageService<K: Ord, S> {
    network: NetworkStorage<K, S>,
    nodes: BTreeMap<NodeId, ProviderState<K, S>>,
    globals: Vec<ProviderState<K, S>>,
    watchers: BTreeMap<WatcherId, StackWatcher<K>>,
    cached: Counts<K>,
    previous: Counts<K>,
    dirty: bool,
}
impl<K: Ord, S> Default for StorageService<K, S> {
    fn default() -> Self {
        Self {
            network: NetworkStorage::default(),
            nodes: BTreeMap::new(),
            globals: vec![],
            watchers: BTreeMap::new(),
            cached: Counts::new(),
            previous: Counts::new(),
            dirty: true,
        }
    }
}
impl<K: Ord + Clone, S> StorageService<K, S> {
    pub fn inventory(&self) -> &NetworkStorage<K, S> {
        &self.network
    }
    pub fn add_node(&mut self, node: NodeId, provider: Provider<K, S>) -> Result<(), &'static str> {
        if self.nodes.contains_key(&node) {
            return Err("duplicate node registration");
        }
        let state = self
            .nodes
            .entry(node)
            .or_insert_with(|| ProviderState::new(provider));
        state.mount(&self.network)
    }
    pub fn remove_node(&mut self, node: NodeId) {
        if let Some(watcher) = self.watchers.remove(&node) {
            watcher.destroy();
        }
        if let Some(mut state) = self.nodes.remove(&node) {
            state.unmount(&self.network);
        }
    }
    pub fn refresh_node(&mut self, node: NodeId) -> Result<(), &'static str> {
        self.nodes
            .get_mut(&node)
            .ok_or("node has no storage provider")?
            .update(&self.network)
    }
    pub fn add_global(&mut self, provider: Provider<K, S>) -> Result<(), &'static str> {
        if self
            .globals
            .iter()
            .any(|s| Arc::ptr_eq(&s.provider, &provider))
        {
            return Err("duplicate global provider registration");
        }
        self.globals.push(ProviderState::new(provider));
        self.globals.last_mut().unwrap().mount(&self.network)
    }
    pub fn remove_global(&mut self, provider: &Provider<K, S>) {
        if let Some(i) = self
            .globals
            .iter()
            .position(|s| Arc::ptr_eq(&s.provider, provider))
        {
            self.globals.remove(i).unmount(&self.network);
        }
    }
    pub fn refresh_global(&mut self, provider: &Provider<K, S>) -> Result<(), &'static str> {
        self.globals
            .iter_mut()
            .find(|s| Arc::ptr_eq(&s.provider, provider))
            .ok_or("global provider not registered")?
            .update(&self.network)
    }
    pub fn add_watcher(&mut self, node: NodeId) -> Result<StackWatcher<K>, &'static str> {
        if self.watchers.contains_key(&node) {
            return Err("duplicate watcher node");
        }
        let watcher = StackWatcher::default();
        self.watchers.insert(node, watcher.clone());
        Ok(watcher)
    }
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }
    pub fn end_tick(&mut self, notify: &mut impl FnMut(&mut Self, WatcherId, &K, i64)) {
        if self.watchers.values().all(StackWatcher::empty) {
            self.dirty = true;
        } else {
            self.update(notify);
        }
    }
    pub fn cached(
        &mut self,
        notify: &mut impl FnMut(&mut Self, WatcherId, &K, i64),
    ) -> &mut Counts<K> {
        if self.dirty {
            self.update(notify);
        }
        &mut self.cached
    }
    fn post(
        &mut self,
        key: &K,
        amount: i64,
        notify: &mut impl FnMut(&mut Self, WatcherId, &K, i64),
    ) {
        let interested: Vec<_> = self
            .watchers
            .iter()
            .filter_map(|(id, watcher)| {
                watcher
                    .interests
                    .lock()
                    .unwrap()
                    .keys
                    .contains(key)
                    .then_some(*id)
            })
            .collect();
        for id in interested {
            notify(self, id, key, amount);
        }
        // A watcher registered both for the key and for all keys is notified
        // twice, exactly as the two InterestManager collections in upstream.
        let all: Vec<_> = self
            .watchers
            .iter()
            .filter_map(|(id, watcher)| watcher.interests.lock().unwrap().all.then_some(*id))
            .collect();
        for id in all {
            notify(self, id, key, amount);
        }
    }
    fn update(&mut self, notify: &mut impl FnMut(&mut Self, WatcherId, &K, i64)) {
        self.dirty = false;
        self.cached.clear();
        self.network.available(&mut self.cached);
        let current = self.cached.clone();
        for (key, amount) in current {
            if amount != self.previous.get(&key).copied().unwrap_or(0) {
                self.post(&key, amount, notify);
            }
        }
        let previous_keys: Vec<_> = self.previous.keys().cloned().collect();
        for key in previous_keys {
            let amount = self.cached.get(&key).copied().unwrap_or(0);
            if amount == 0 {
                self.post(&key, amount, notify);
            }
        }
        self.previous.clone_from(&self.cached);
    }
}

#[cfg(test)]
mod tests {
    use super::super::ae2_mounts::{Storage, add_count};
    use super::*;
    struct Bin(i64);
    impl Storage<u32, ()> for Bin {
        fn available(&mut self, out: &mut Counts<u32>) {
            if self.0 != 0 {
                add_count(out, 1, self.0);
            }
        }
    }
    struct ProviderList {
        inventories: Vec<Inventory<u32, ()>>,
        enabled: bool,
        priority: i32,
    }
    impl StorageProvider<u32, ()> for ProviderList {
        fn mount(&mut self, mounts: &mut StorageMounts<'_, u32, ()>) -> Result<(), &'static str> {
            if self.enabled {
                for i in &self.inventories {
                    mounts.mount(i.clone(), self.priority)?;
                }
            }
            Ok(())
        }
    }
    type Fixture = (
        StorageService<u32, ()>,
        Arc<Mutex<Bin>>,
        Arc<Mutex<ProviderList>>,
    );
    fn fixture() -> Fixture {
        let bin = Arc::new(Mutex::new(Bin(10)));
        let provider = Arc::new(Mutex::new(ProviderList {
            inventories: vec![bin.clone()],
            enabled: true,
            priority: 0,
        }));
        (StorageService::default(), bin, provider)
    }
    fn ignore(_: &mut StorageService<u32, ()>, _: WatcherId, _: &u32, _: i64) {}
    #[test]
    fn provider_refresh_unmounts_before_mounting_and_respects_offline_state() {
        let (mut service, _, provider) = fixture();
        service.add_node(1, provider.clone()).unwrap();
        assert_eq!(service.cached(&mut ignore)[&1], 10);
        provider.lock().unwrap().enabled = false;
        service.refresh_node(1).unwrap();
        // Mount changes do not themselves invalidate the public cached list.
        assert_eq!(service.cached(&mut ignore)[&1], 10);
        service.end_tick(&mut ignore);
        assert!(service.cached(&mut ignore).is_empty());
        provider.lock().unwrap().enabled = true;
        service.refresh_node(1).unwrap();
        service.end_tick(&mut ignore);
        assert_eq!(service.cached(&mut ignore)[&1], 10);
        service.remove_node(1);
        service.end_tick(&mut ignore);
        assert!(service.cached(&mut ignore).is_empty());
        assert!(service.refresh_node(1).is_err());
    }
    #[test]
    fn duplicate_inventory_and_global_provider_are_rejected_by_identity() {
        let (mut service, bin, provider) = fixture();
        let erased: Provider<_, _> = provider.clone();
        service.add_global(erased.clone()).unwrap();
        assert!(service.add_global(erased.clone()).is_err());
        service.remove_global(&erased);
        assert!(service.refresh_global(&erased).is_err());
        provider.lock().unwrap().inventories.push(bin);
        assert!(service.add_node(1, provider).is_err());
        service.remove_node(1);
        let mut out = Counts::new();
        service.inventory().available(&mut out);
        assert!(out.is_empty());
    }
    #[test]
    fn private_previous_counts_survive_accidental_public_cache_mutation() {
        let (mut service, bin, provider) = fixture();
        service.add_node(1, provider).unwrap();
        let watcher = service.add_watcher(2).unwrap();
        watcher.watch_all(true);
        let mut events = vec![];
        service.cached(&mut |_, id, key, amount| events.push((id, *key, amount)));
        assert_eq!(events, vec![(2, 1, 10)]);
        events.clear();
        service.cached(&mut ignore).insert(1, 999);
        service.end_tick(&mut |_, id, key, amount| events.push((id, *key, amount)));
        assert!(events.is_empty());
        bin.lock().unwrap().0 = 7;
        service.end_tick(&mut |_, id, key, amount| events.push((id, *key, amount)));
        assert_eq!(events, vec![(2, 1, 7)]);
    }
    #[test]
    fn watch_all_and_key_interests_both_deliver_and_removed_keys_report_zero() {
        let (mut service, bin, provider) = fixture();
        service.add_node(1, provider).unwrap();
        let watcher = service.add_watcher(2).unwrap();
        watcher.add(1);
        watcher.add(1);
        watcher.watch_all(true);
        let mut events = vec![];
        service.end_tick(&mut |_, _, _, n| events.push(n));
        assert_eq!(events, vec![10, 10]);
        events.clear();
        bin.lock().unwrap().0 = 0;
        service.end_tick(&mut |_, _, _, n| events.push(n));
        assert_eq!(events, vec![0, 0]);
        watcher.reset();
        bin.lock().unwrap().0 = 9;
        service.end_tick(&mut |_, _, _, n| events.push(n));
        assert_eq!(events, vec![0, 0]);
    }
    #[test]
    fn old_watcher_handle_cannot_reactivate_after_node_leaves_grid() {
        let (mut service, _, provider) = fixture();
        service.add_node(1, provider).unwrap();
        let old = service.add_watcher(2).unwrap();
        old.watch_all(true);
        old.add(1);
        service.remove_node(2);
        old.add(1);
        old.watch_all(true);
        assert!(old.empty());
        let new = service.add_watcher(2).unwrap();
        assert!(new.empty());
        new.add(1);
        assert!(!new.empty());
    }
    #[test]
    fn notifications_can_read_new_public_cache_without_recursively_refreshing() {
        let (mut service, _, provider) = fixture();
        service.add_node(1, provider).unwrap();
        service.add_watcher(2).unwrap().watch_all(true);
        let mut events = 0;
        service.end_tick(&mut |s, _, key, amount| {
            assert_eq!(s.cached(&mut ignore)[key], amount);
            events += 1;
        });
        assert_eq!(events, 1);
    }
}
