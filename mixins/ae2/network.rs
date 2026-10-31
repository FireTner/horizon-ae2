// SPDX-License-Identifier: LGPL-3.0-or-later
// World-independent binding of AE2 full-block nodes to topology + energy services.
// This grows alongside the complete port; CableBus parts and other services are
// not approximated by these four currently implemented node kinds.

use super::ae2_grid::{ChannelMode, ConnectionId, Direction, Flags, Graph, NodeId, NodeSpec};
use super::ae2_power::{EnergyNode, EnergyService, EnergySystem, ServiceId, Store};
use std::collections::{BTreeMap, BTreeSet};
pub type Pos = [i32; 3];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Charger,
    EnergyCell,
    DenseEnergyCell,
    CreativeEnergyCell,
    Drive,
}
impl Kind {
    pub fn from_block(id: &str) -> Option<Self> {
        match id {
            "ae2:charger" => Some(Self::Charger),
            "ae2:energy_cell" => Some(Self::EnergyCell),
            "ae2:dense_energy_cell" => Some(Self::DenseEnergyCell),
            "ae2:creative_energy_cell" => Some(Self::CreativeEnergyCell),
            "ae2:drive" => Some(Self::Drive),
            _ => None,
        }
    }
    pub fn block(self) -> &'static str {
        match self {
            Self::Charger => "ae2:charger",
            Self::EnergyCell => "ae2:energy_cell",
            Self::DenseEnergyCell => "ae2:dense_energy_cell",
            Self::CreativeEnergyCell => "ae2:creative_energy_cell",
            Self::Drive => "ae2:drive",
        }
    }
    pub fn capacity(self) -> f64 {
        match self {
            Self::Charger => 1600.0,
            Self::EnergyCell => 200000.0,
            Self::DenseEnergyCell => 1600000.0,
            Self::CreativeEnergyCell => super::ae2_energy::MAX_MAXIMUM,
            Self::Drive => 0.0,
        }
    }
    fn energy_node(self, amount: f64) -> EnergyNode {
        let store = match self {
            Self::Charger | Self::Drive => None,
            Self::EnergyCell => Some(Store::finite(
                amount.clamp(0.0, self.capacity()),
                self.capacity(),
                200,
            )),
            Self::DenseEnergyCell => Some(Store::finite(
                amount.clamp(0.0, self.capacity()),
                self.capacity(),
                1600,
            )),
            Self::CreativeEnergyCell => Some(Store::creative()),
        };
        EnergyNode {
            idle: if self == Self::Drive { 2.0 } else { 0.0 },
            store,
            passive_rate: None,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub pos: Pos,
    pub kind: Kind,
    pub front: Direction,
    pub stored: f64,
    pub saved_buffer: f64,
}
impl Snapshot {
    fn exposes(self, direction: Direction) -> bool {
        !matches!(self.kind, Kind::Charger | Kind::Drive) || self.front != direction
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Persisted {
    pub pos: Pos,
    pub kind: Kind,
    pub stored: Option<f64>,
    pub buffer_share: f64,
    pub powered: bool,
}
#[derive(Clone, Debug)]
struct Node {
    id: NodeId,
    snapshot: Snapshot,
    service: Option<ServiceId>,
}

#[derive(Clone, Debug, Default)]
pub struct Network {
    graph: Graph,
    energy: EnergySystem,
    nodes: BTreeMap<Pos, Node>,
    connections: BTreeMap<(NodeId, NodeId), ConnectionId>,
    pivots: BTreeMap<ServiceId, NodeId>,
    channel_power: BTreeMap<ServiceId, f64>,
    next_service: ServiceId,
}
impl Network {
    fn new_service(&mut self, pivot: NodeId) -> ServiceId {
        let id = self.next_service;
        self.next_service += 1;
        self.energy
            .add_service(id, EnergyService::default())
            .expect("unique service ID");
        self.pivots.insert(id, pivot);
        id
    }
    /// Reconcile loaded nodes. A removed unloaded node's last persisted proxy.e
    /// already carries its share; active services deduct it proportionally.
    pub fn sync(&mut self, snapshots: Vec<Snapshot>) {
        let incoming: BTreeMap<_, _> = snapshots.into_iter().map(|s| (s.pos, s)).collect();
        let mut changed = false;
        let gone: Vec<_> = self
            .nodes
            .iter()
            .filter(|(p, n)| incoming.get(*p).is_none_or(|s| s.kind != n.snapshot.kind))
            .map(|(p, _)| *p)
            .collect();
        for pos in gone {
            let n = self.nodes.remove(&pos).unwrap();
            self.graph.remove_node(n.id);
            if let Some(service) = n.service {
                self.energy.remove_node(service, n.id);
            }
            self.connections
                .retain(|(a, b), _| *a != n.id && *b != n.id);
            changed = true;
        }
        for (pos, snapshot) in incoming {
            match self.nodes.get_mut(&pos) {
                Some(node) => {
                    if node.snapshot.front != snapshot.front {
                        changed = true;
                    }
                    if node.snapshot.stored.to_bits() != snapshot.stored.to_bits()
                        && let Some(service) = node.service
                        && snapshot.kind != Kind::Charger
                    {
                        let _ = self
                            .energy
                            .service_mut(service)
                            .unwrap()
                            .set_storage_amount(node.id, snapshot.stored);
                    }
                    node.snapshot = snapshot;
                }
                None => {
                    let id = self.graph.add_node(NodeSpec {
                        flags: if snapshot.kind == Kind::Drive {
                            Flags::REQUIRE_CHANNEL
                        } else {
                            Flags::default()
                        },
                        ..Default::default()
                    });
                    self.nodes.insert(
                        pos,
                        Node {
                            id,
                            snapshot,
                            service: None,
                        },
                    );
                    changed = true;
                }
            }
        }
        let mut desired = BTreeMap::new();
        for (pos, node) in &self.nodes {
            for direction in Direction::ALL {
                if !node.snapshot.exposes(direction) {
                    continue;
                }
                let delta = direction.offset();
                let neighbor = std::array::from_fn(|i| pos[i] + delta[i]);
                if let Some(other) = self.nodes.get(&neighbor)
                    && node.id < other.id
                    && other.snapshot.exposes(direction.opposite())
                {
                    desired.insert((node.id, other.id), direction);
                }
            }
        }
        let removed: Vec<_> = self
            .connections
            .keys()
            .filter(|key| !desired.contains_key(key))
            .copied()
            .collect();
        for key in removed {
            self.graph
                .disconnect(self.connections.remove(&key).unwrap());
            changed = true;
        }
        for (key, direction) in desired {
            if let std::collections::btree_map::Entry::Vacant(entry) = self.connections.entry(key) {
                entry.insert(
                    self.graph
                        .connect(key.0, key.1, Some(direction))
                        .expect("unique adjacent connection"),
                );
                changed = true;
            }
        }
        if !changed {
            return;
        }
        // On a split the component containing the old pivot retains its service.
        // On a merge the larger existing grid wins (all current kinds priority 0).
        // Equal-size tie order and full Grid service/event identity remain a port gap.
        for (service, pivot) in &mut self.pivots {
            if !self
                .nodes
                .values()
                .any(|n| n.id == *pivot && n.service == Some(*service))
                && let Some(first) = self
                    .nodes
                    .values()
                    .filter(|n| n.service == Some(*service))
                    .map(|n| n.id)
                    .min()
            {
                *pivot = first;
            }
        }
        let reports = self.graph.calculate(ChannelMode::Default);
        let old_sizes: BTreeMap<_, _> = self
            .pivots
            .keys()
            .map(|s| {
                (
                    *s,
                    self.nodes
                        .values()
                        .filter(|n| n.service == Some(*s))
                        .count(),
                )
            })
            .collect();
        let mut assignments = BTreeMap::new();
        self.channel_power.clear();
        for report in reports {
            let candidates: Vec<_> = self
                .pivots
                .iter()
                .filter(|(_, pivot)| report.nodes.contains(pivot))
                .map(|(s, _)| *s)
                .collect();
            let winner = candidates
                .into_iter()
                .max_by_key(|s| (old_sizes[s], std::cmp::Reverse(*s)))
                .unwrap_or_else(|| self.new_service(*report.nodes.first().unwrap()));
            self.channel_power
                .insert(winner, report.channel_power_usage());
            for id in report.nodes {
                assignments.insert(id, winner);
            }
        }
        let moving: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.service != Some(assignments[&n.id]))
            .map(|(p, _)| *p)
            .collect();
        let mut transfers = BTreeMap::new();
        for pos in &moving {
            let node = &self.nodes[pos];
            if let Some(service) = node.service {
                let removed = self
                    .energy
                    .remove_node(service, node.id)
                    .expect("registered energy node");
                transfers.insert(*pos, removed);
            }
        }
        for pos in moving {
            let node = self.nodes.get_mut(&pos).unwrap();
            let service = assignments[&node.id];
            let (spec, buffer) = match transfers.remove(&pos) {
                Some(t) => (t.node, t.saved_buffer),
                None => (
                    node.snapshot.kind.energy_node(node.snapshot.stored),
                    node.snapshot.saved_buffer,
                ),
            };
            self.energy
                .service_mut(service)
                .unwrap()
                .add_node(node.id, spec, buffer)
                .expect("valid full-block energy node");
            node.service = Some(service);
        }
        let used: BTreeSet<_> = self.nodes.values().filter_map(|n| n.service).collect();
        let unused: Vec<_> = self
            .pivots
            .keys()
            .filter(|s| !used.contains(s))
            .copied()
            .collect();
        for service in unused {
            self.energy.remove_service(service);
            self.pivots.remove(&service);
        }
    }
    pub fn begin_tick(&mut self) {
        self.energy.begin_tick();
    }
    pub fn extract(&mut self, pos: Pos, amount: f64, multiplier: f64, commit: bool) -> f64 {
        let Some(service) = self.nodes.get(&pos).and_then(|n| n.service) else {
            return 0.0;
        };
        self.energy.extract(service, amount, multiplier, commit)
    }
    pub fn end_tick(&mut self) -> Vec<Persisted> {
        for (service, channel_power) in self.channel_power.clone() {
            self.energy.end_tick(service, channel_power, 1.0);
        }
        self.nodes
            .values_mut()
            .map(|node| {
                let service = self.energy.service(node.service.unwrap()).unwrap();
                let stored = service.store(node.id).map(Store::amount);
                if let Some(value) = stored {
                    node.snapshot.stored = value;
                }
                node.snapshot.saved_buffer = service.node_energy_share();
                Persisted {
                    pos: node.snapshot.pos,
                    kind: node.snapshot.kind,
                    stored,
                    buffer_share: node.snapshot.saved_buffer,
                    powered: service.is_powered(),
                }
            })
            .collect()
    }
    pub fn is_powered(&self, pos: Pos) -> bool {
        self.nodes
            .get(&pos)
            .and_then(|n| n.service)
            .and_then(|s| self.energy.service(s))
            .is_some_and(EnergyService::is_powered)
    }
    pub fn node_service(&self, pos: Pos) -> Option<(ServiceId, NodeId)> {
        let node = self.nodes.get(&pos)?;
        Some((node.service?, node.id))
    }
    pub fn set_idle(&mut self, pos: Pos, idle: f64) -> Result<(), &'static str> {
        let (service, node) = self.node_service(pos).ok_or("unknown network node")?;
        self.energy
            .service_mut(service)
            .unwrap()
            .set_idle(node, idle)
    }
    pub fn is_online(&self, pos: Pos) -> bool {
        self.is_powered(pos)
            && self
                .nodes
                .get(&pos)
                .and_then(|n| self.graph.node(n.id))
                .is_some_and(|n| n.meets_channel_requirements())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot(pos: Pos, kind: Kind, stored: f64) -> Snapshot {
        Snapshot {
            pos,
            kind,
            front: Direction::North,
            stored,
            saved_buffer: 0.0,
        }
    }
    #[test]
    fn finite_cell_drives_charger_only_through_an_exposed_face() {
        let mut n = Network::default();
        let charger = snapshot([0, 0, 0], Kind::Charger, 0.0);
        let back = snapshot([0, 0, 1], Kind::EnergyCell, 2000.0);
        n.sync(vec![charger, back]);
        n.begin_tick();
        assert_eq!(n.extract([0, 0, 0], 800.0, 1.0, true), 800.0);
        let saved = n.end_tick();
        assert_eq!(
            saved
                .iter()
                .find(|s| s.kind == Kind::EnergyCell)
                .unwrap()
                .stored,
            Some(1200.0)
        );
        let mut front = back;
        front.pos = [0, 0, -1];
        n.sync(vec![charger, front]);
        n.begin_tick();
        assert_eq!(n.extract([0, 0, 0], 800.0, 1.0, true), 0.0);
    }
    #[test]
    fn merge_and_split_do_not_duplicate_or_erase_buffer_energy() {
        let mut n = Network::default();
        let mut a = snapshot([0, 0, 0], Kind::Charger, 0.0);
        a.saved_buffer = 10.0;
        let mut b = snapshot([2, 0, 0], Kind::Charger, 0.0);
        b.saved_buffer = 20.0;
        n.sync(vec![a, b]);
        n.begin_tick();
        let bridge = snapshot([1, 0, 0], Kind::Charger, 0.0);
        n.sync(vec![a, b, bridge]);
        let saved = n.end_tick();
        assert!((saved.iter().map(|s| s.buffer_share).sum::<f64>() - 30.0).abs() < 1e-9);
        // Removing a real node deducts its 1/N share, like GridEnergyStorage.
        a.saved_buffer = 10.0;
        b.saved_buffer = 10.0;
        n.sync(vec![a, b]);
        let saved = n.end_tick();
        assert!((saved.iter().map(|s| s.buffer_share).sum::<f64>() - 20.0).abs() < 1e-9);
    }
    #[test]
    fn unloaded_reload_restores_only_its_saved_share() {
        let mut n = Network::default();
        let mut a = snapshot([0, 0, 0], Kind::Charger, 0.0);
        a.saved_buffer = 17.0;
        n.sync(vec![a]);
        let saved = n.end_tick();
        n.sync(vec![]);
        a.saved_buffer = saved[0].buffer_share;
        n.sync(vec![a]);
        assert_eq!(n.extract(a.pos, 100.0, 1.0, true), 17.0);
    }
    #[test]
    fn external_nbt_energy_change_is_applied_once_and_stable_ticks_do_not_reset_it() {
        let mut n = Network::default();
        let a = snapshot([0, 0, 0], Kind::EnergyCell, 1000.0);
        n.sync(vec![a]);
        n.extract(a.pos, 100.0, 1.0, true);
        n.sync(vec![a]); // post-tick: NBT still has the old value until persist
        let saved = n.end_tick();
        let mut a = a;
        a.stored = saved[0].stored.unwrap();
        n.sync(vec![a]);
        assert_eq!(n.extract(a.pos, 1000.0, 1.0, true), 900.0);
        a.stored = 777.0;
        n.sync(vec![a]);
        assert_eq!(n.extract(a.pos, 1000.0, 1.0, true), 777.0);
    }
    #[test]
    fn replacing_a_block_destroys_old_energy_node_and_uses_new_kind() {
        let mut n = Network::default();
        let a = snapshot([0, 0, 0], Kind::CreativeEnergyCell, 0.0);
        n.sync(vec![a]);
        assert_eq!(n.extract(a.pos, 1e6, 1.0, true), 1e6);
        n.sync(vec![snapshot(a.pos, Kind::EnergyCell, 10.0)]);
        assert_eq!(n.extract(a.pos, 100.0, 1.0, true), 10.0);
    }
    #[test]
    fn drives_consume_real_idle_energy_and_require_channels() {
        let mut n = Network::default();
        let a = snapshot([0, 0, 0], Kind::Drive, 0.0);
        let source = snapshot([0, 0, 1], Kind::EnergyCell, 1000.0);
        n.sync(vec![a, source]);
        n.set_idle(a.pos, 2.5).unwrap();
        for _ in 0..40 {
            n.begin_tick();
            n.end_tick();
        }
        assert!(n.is_online(a.pos));
        // One ad-hoc channel across two nodes costs another 2/128 AE/tick.
        assert_eq!(
            n.extract(a.pos, 1000.0, 1.0, false),
            1000.0 - 40.0 * (2.5 + 2.0 / 128.0)
        );
        let mut drives = vec![snapshot([0, 0, 1], Kind::CreativeEnergyCell, 0.0)];
        for x in 0..9 {
            drives.push(snapshot([x, 0, 0], Kind::Drive, 0.0));
        }
        n.sync(drives.clone());
        for _ in 0..40 {
            n.begin_tick();
            n.end_tick();
        }
        assert!(n.is_powered(a.pos));
        assert!(!n.is_online(a.pos));
        drives.pop();
        n.sync(drives);
        n.begin_tick();
        n.end_tick();
        assert!(n.is_online(a.pos));
    }
}
