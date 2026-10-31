// SPDX-License-Identifier: LGPL-3.0-or-later
// DriveBlockEntity / DriveWatcher basic-cell rules from pinned AE2.
// Copyright AlgorithmX2 / TeamAppliedEnergistics; 2026 Horizon port contributors.
// ItemStack slots, persistence, node lifecycle and GUI belong to the adapter.

use super::ae2_backend::CellBackend;
use super::ae2_mounts::{Counts, Storage, add_count};
use super::ae2_service::{StorageMounts, StorageProvider};
use super::ae2_storage::{CellKey, CellState};
use std::sync::{Arc, Mutex};
pub const SLOTS: usize = 10;

pub struct DriveCell<K> {
    pub cell: CellBackend<K>,
    pub dirty: bool,
    pub status_changed: bool,
    old_status: CellState,
}
impl<K: CellKey> DriveCell<K> {
    pub fn new(cell: impl Into<CellBackend<K>>) -> Self {
        let cell = cell.into();
        let old_status = cell.state();
        Self {
            cell,
            old_status,
            dirty: false,
            status_changed: false,
        }
    }
    fn changed(&mut self) {
        self.dirty = true;
        let state = self.cell.state();
        if state != self.old_status {
            self.status_changed = true;
            self.old_status = state;
        }
    }
}
impl<K: CellKey + Send, S> Storage<K, S> for DriveCell<K> {
    fn preferred(&mut self, key: &K, _: &S) -> bool {
        // DriveWatcher inherits MEInventoryHandler's default partition list:
        // an existing equal stack makes the cell preferred, via SIMULATE.
        self.cell.extract(key, 1, false) > 0
    }
    fn insert(&mut self, key: &K, amount: u64, commit: bool, _: &S) -> u64 {
        let inserted = self.cell.insert(key.clone(), amount, commit);
        if commit && inserted > 0 {
            self.changed();
        }
        inserted
    }
    fn extract(&mut self, key: &K, amount: u64, commit: bool, _: &S) -> u64 {
        let extracted = self.cell.extract(key, amount, commit);
        if commit && extracted > 0 {
            self.changed();
        }
        extracted
    }
    fn available(&mut self, out: &mut Counts<K>) {
        for (key, amount) in self.cell.stacks() {
            add_count(out, key.clone(), *amount as i64);
        }
    }
}
pub type Slot<K> = Option<Arc<Mutex<DriveCell<K>>>>;
pub struct Drive<K> {
    pub slots: [Slot<K>; SLOTS],
    pub priority: i32,
    pub online: bool,
}
impl<K> Default for Drive<K> {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| None),
            priority: 0,
            online: false,
        }
    }
}
impl<K: CellKey> Drive<K> {
    pub fn idle_drain(&self) -> f64 {
        2.0 + self
            .slots
            .iter()
            .flatten()
            .map(|cell| cell.lock().unwrap().cell.idle_drain())
            .sum::<f64>()
    }
    pub fn packed_state(&self) -> u32 {
        let mut packed = if self.online { 1 << 31 } else { 0 };
        for (slot, cell) in self.slots.iter().enumerate() {
            let ordinal = cell
                .as_ref()
                .map_or(0, |cell| match cell.lock().unwrap().cell.state() {
                    CellState::Empty => 1,
                    CellState::NotEmpty => 2,
                    CellState::TypesFull => 3,
                    CellState::Full => 4,
                });
            packed |= ordinal << (slot * 3);
        }
        packed
    }
}
impl<K: CellKey + Send + 'static, S> StorageProvider<K, S> for Drive<K> {
    fn mount(&mut self, mounts: &mut StorageMounts<'_, K, S>) -> Result<(), &'static str> {
        if self.online {
            for inventory in self.slots.iter().flatten() {
                mounts.mount(inventory.clone(), self.priority)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        ae2_keys::{Key, KeyType},
        ae2_service::StorageService,
        ae2_storage::{Cell, Tier},
    };
    use super::*;
    fn key(id: &str) -> Key {
        Key::new(KeyType::Item, id.to_owned(), vec![], Default::default()).unwrap()
    }
    fn cell(tier: Tier) -> Arc<Mutex<DriveCell<Key>>> {
        Arc::new(Mutex::new(DriveCell::new(Cell::new(tier, KeyType::Item))))
    }
    #[test]
    fn creative_drive_mount_has_zero_disk_drain_and_unlimited_configured_extraction() {
        let iron = key("minecraft:iron_ingot");
        let disk = Arc::new(Mutex::new(DriveCell::new(
            super::super::ae2_backend::CreativeCell::new(std::collections::BTreeSet::from([
                iron.clone()
            ])),
        )));
        let mut drive = Drive::<Key> {
            online: true,
            ..Default::default()
        };
        drive.slots[0] = Some(disk.clone());
        assert_eq!(drive.idle_drain(), 2.0);
        assert_eq!(drive.packed_state() & 7, 3);
        let drive = Arc::new(Mutex::new(drive));
        let mut service = StorageService::<Key, ()>::default();
        service.add_node(1, drive).unwrap();
        let inventory = service.inventory();
        for _ in 0..2 {
            assert_eq!(
                inventory.extract(&iron, 1_000_000_000_000, true, &()),
                1_000_000_000_000
            );
        }
        assert_eq!(
            inventory.insert(&iron, 1_000_000_000_000, true, &()),
            1_000_000_000_000
        );
        assert_eq!(
            inventory.extract(&key("minecraft:gold_ingot"), 1, true, &()),
            0
        );
        assert_eq!(disk.lock().unwrap().cell.stacks()[&iron], i32::MAX as u64);
    }
    #[test]
    fn only_online_drives_mount_and_disks_keep_their_contents_when_unmounted() {
        let mut service = StorageService::<Key, ()>::default();
        let drive = Arc::new(Mutex::new(Drive::default()));
        let disk = cell(Tier::K1);
        drive.lock().unwrap().slots[0] = Some(disk.clone());
        service.add_node(1, drive.clone()).unwrap();
        let inventory = service.inventory().clone();
        assert_eq!(
            inventory.insert(&key("minecraft:iron_ingot"), 10, true, &()),
            0
        );
        drive.lock().unwrap().online = true;
        service.refresh_node(1).unwrap();
        assert_eq!(
            inventory.insert(&key("minecraft:iron_ingot"), 10000, false, &()),
            8128
        );
        assert!(disk.lock().unwrap().cell.stacks().is_empty());
        assert_eq!(
            inventory.insert(&key("minecraft:iron_ingot"), 100, true, &()),
            100
        );
        drive.lock().unwrap().online = false;
        service.refresh_node(1).unwrap();
        assert_eq!(
            inventory.extract(&key("minecraft:iron_ingot"), 100, true, &()),
            0
        );
        assert_eq!(disk.lock().unwrap().cell.stored_count(), 100);
        drive.lock().unwrap().online = true;
        service.refresh_node(1).unwrap();
        assert_eq!(
            inventory.extract(&key("minecraft:iron_ingot"), 100, true, &()),
            100
        );
    }
    #[test]
    fn idle_draw_and_client_state_cover_all_ten_slots() {
        let mut drive: Drive<Key> = Drive::default();
        assert_eq!(drive.idle_drain(), 2.0);
        drive.slots[0] = Some(cell(Tier::K1));
        drive.slots[9] = Some(cell(Tier::K256));
        assert_eq!(drive.idle_drain(), 5.0);
        assert_eq!(drive.packed_state(), 1 | (1 << 27));
        drive.online = true;
        assert_eq!(drive.packed_state(), (1 << 31) | 1 | (1 << 27));
    }
    #[test]
    fn mounted_priority_changes_reorder_cell_selection_without_copying_disks() {
        let mut service = StorageService::<Key, ()>::default();
        let a = Arc::new(Mutex::new(Drive::default()));
        let b = Arc::new(Mutex::new(Drive::default()));
        let ca = cell(Tier::K1);
        let cb = cell(Tier::K1);
        a.lock().unwrap().slots[0] = Some(ca.clone());
        b.lock().unwrap().slots[0] = Some(cb.clone());
        a.lock().unwrap().online = true;
        b.lock().unwrap().online = true;
        b.lock().unwrap().priority = 10;
        service.add_node(1, a.clone()).unwrap();
        service.add_node(2, b.clone()).unwrap();
        let net = service.inventory().clone();
        let iron = key("minecraft:iron_ingot");
        net.insert(&iron, 10, true, &());
        assert_eq!(cb.lock().unwrap().cell.stored_count(), 10);
        a.lock().unwrap().priority = 20;
        service.refresh_node(1).unwrap();
        net.insert(&iron, 5, true, &());
        assert_eq!(ca.lock().unwrap().cell.stored_count(), 5);
        net.extract(&iron, 3, true, &());
        assert_eq!(cb.lock().unwrap().cell.stored_count(), 7);
        assert_eq!(ca.lock().unwrap().cell.stored_count(), 5);
    }
    #[test]
    fn preferred_cells_and_status_changes_follow_drive_watcher() {
        let mut drive: Drive<Key> = Drive::default();
        let a = cell(Tier::K1);
        let b = cell(Tier::K1);
        let iron = key("minecraft:iron_ingot");
        b.lock().unwrap().insert(&iron, 10, true, &());
        drive.slots[0] = Some(a.clone());
        drive.slots[1] = Some(b.clone());
        drive.online = true;
        let mut service = StorageService::<Key, ()>::default();
        service.add_node(1, Arc::new(Mutex::new(drive))).unwrap();
        service.inventory().insert(&iron, 5, true, &());
        assert_eq!(a.lock().unwrap().cell.stored_count(), 0);
        assert_eq!(b.lock().unwrap().cell.stored_count(), 15);
        assert!(b.lock().unwrap().status_changed);
        let mut b = b.lock().unwrap();
        b.status_changed = false;
        b.extract(&iron, 1, true, &());
        assert!(!b.status_changed);
        b.extract(&iron, 14, false, &());
        assert!(!b.status_changed);
        b.extract(&iron, 14, true, &());
        assert!(b.status_changed);
        assert!(b.dirty);
    }
}
