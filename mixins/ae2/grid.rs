// SPDX-License-Identifier: LGPL-3.0-or-later
// Port of ControllerValidator, PathingCalculation, GridNode's pathing methods,
// GridConnection's routing/counting, and PathingService's ad-hoc rules.
// This module is the topology/pathing core, not the world/grid-service adapter.
// Copyright AlgorithmX2 / TeamAppliedEnergistics; 2026 Horizon port contributors.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ops::BitOr;
pub type NodeId = u64;
pub type ConnectionId = u64;
pub type Pos = [i32; 3];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags(u8);
impl Flags {
    pub fn from_bits(bits: u8) -> Option<Self> {
        (bits & !127 == 0).then_some(Self(bits))
    }
    pub const REQUIRE_CHANNEL: Self = Self(1);
    pub const COMPRESSED_CHANNEL: Self = Self(2);
    pub const CANNOT_CARRY: Self = Self(4);
    pub const CANNOT_CARRY_COMPRESSED: Self = Self(8);
    pub const DENSE_CAPACITY: Self = Self(16);
    pub const MULTIBLOCK: Self = Self(32);
    pub const PREFERRED: Self = Self(64);
    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }
}
impl BitOr for Flags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChannelMode {
    Infinite,
    #[default]
    Default,
    X2,
    X3,
    X4,
}
impl ChannelMode {
    pub fn factor(self) -> u32 {
        match self {
            Self::Infinite => 0,
            Self::Default => 1,
            Self::X2 => 2,
            Self::X3 => 3,
            Self::X4 => 4,
        }
    }
    pub fn ad_hoc_limit(self) -> u32 {
        if self == Self::Infinite {
            i32::MAX as u32
        } else {
            8 * self.factor()
        }
    }
    pub fn connection_limit(self) -> u32 {
        if self == Self::Infinite {
            i32::MAX as u32
        } else {
            32 * self.factor()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}
impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }
    pub fn offset(self) -> Pos {
        match self {
            Self::Down => [0, -1, 0],
            Self::Up => [0, 1, 0],
            Self::North => [0, 0, -1],
            Self::South => [0, 0, 1],
            Self::West => [-1, 0, 0],
            Self::East => [1, 0, 0],
        }
    }
    pub const ALL: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];
}

#[derive(Clone, Debug, Default)]
pub struct NodeSpec {
    pub flags: Flags,
    pub controller: Option<Pos>,
    pub multiblock: BTreeSet<NodeId>,
}
#[derive(Clone, Debug)]
pub struct Node {
    pub spec: NodeSpec,
    connections: Vec<ConnectionId>,
    used: u32,
    finalized: u32,
    ancestor: Option<NodeId>,
    subtree_max: u32,
    subtree_compressed: bool,
}
impl Node {
    pub fn used_channels(&self) -> u32 {
        self.finalized
    }
    pub fn meets_channel_requirements(&self) -> bool {
        !self.spec.flags.has(Flags::REQUIRE_CHANNEL) || self.finalized > 0
    }
    pub fn max_channels(&self, mode: ChannelMode) -> u32 {
        if self.spec.flags.has(Flags::CANNOT_CARRY) {
            0
        } else if mode == ChannelMode::Infinite {
            i32::MAX as u32
        } else if self.spec.flags.has(Flags::DENSE_CAPACITY) {
            32 * mode.factor()
        } else {
            8 * mode.factor()
        }
    }
    pub fn connections(&self) -> &[ConnectionId] {
        &self.connections
    }
}
#[derive(Clone, Debug)]
pub struct Connection {
    a: NodeId,
    b: NodeId,
    direction: Option<Direction>,
    used: u32,
    finalized: u32,
}
impl Connection {
    pub fn a(&self) -> NodeId {
        self.a
    }
    pub fn b(&self) -> NodeId {
        self.b
    }
    pub fn used_channels(&self) -> u32 {
        self.finalized
    }
    pub fn other(&self, node: NodeId) -> Option<NodeId> {
        if node == self.a {
            Some(self.b)
        } else if node == self.b {
            Some(self.a)
        } else {
            None
        }
    }
    pub fn direction(&self, node: NodeId) -> Option<Direction> {
        self.direction
            .map(|d| if node == self.a { d } else { d.opposite() })
    }
    fn route_towards(&mut self, node: NodeId) {
        self.used = 0;
        if self.b == node {
            std::mem::swap(&mut self.a, &mut self.b);
            self.direction = self.direction.map(Direction::opposite);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerState {
    None,
    Online,
    Conflict,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdHocError {
    TooManyChannels,
    NestedP2P,
}
#[derive(Clone, Debug)]
pub struct GridReport {
    pub nodes: BTreeSet<NodeId>,
    pub controller: ControllerState,
    pub ad_hoc_error: Option<AdHocError>,
    pub channels_in_use: u32,
    pub channels_by_blocks: u64,
    pub changed_nodes: Vec<NodeId>,
    pub changed_connections: Vec<ConnectionId>,
}
impl GridReport {
    pub fn channel_power_usage(&self) -> f64 {
        self.channels_by_blocks as f64 / 128.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct Graph {
    nodes: BTreeMap<NodeId, Node>,
    connections: BTreeMap<ConnectionId, Connection>,
    next_node: NodeId,
    next_connection: ConnectionId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PathItem {
    Node(NodeId),
    Connection(ConnectionId),
}

impl Graph {
    pub fn add_node(&mut self, spec: NodeSpec) -> NodeId {
        let id = self.next_node;
        self.next_node += 1;
        self.nodes.insert(
            id,
            Node {
                spec,
                connections: vec![],
                used: 0,
                finalized: 0,
                ancestor: None,
                subtree_max: 0,
                subtree_compressed: true,
            },
        );
        id
    }
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }
    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }
    pub fn connection(&self, id: ConnectionId) -> Option<&Connection> {
        self.connections.get(&id)
    }
    pub fn connect(
        &mut self,
        a: NodeId,
        b: NodeId,
        direction: Option<Direction>,
    ) -> Result<ConnectionId, &'static str> {
        if a == b {
            return Err("cannot connect node to itself");
        }
        let a_node = self.nodes.get(&a).ok_or("unknown node A")?;
        if !self.nodes.contains_key(&b) {
            return Err("unknown node B");
        }
        if a_node
            .connections
            .iter()
            .any(|id| self.connections[id].other(a) == Some(b))
        {
            return Err("connection already exists");
        }
        let id = self.next_connection;
        self.next_connection += 1;
        self.connections.insert(
            id,
            Connection {
                a,
                b,
                direction,
                used: 0,
                finalized: 0,
            },
        );
        self.nodes.get_mut(&a).unwrap().connections.push(id);
        self.nodes.get_mut(&b).unwrap().connections.push(id);
        Ok(id)
    }
    pub fn disconnect(&mut self, id: ConnectionId) -> bool {
        let Some(c) = self.connections.remove(&id) else {
            return false;
        };
        self.nodes
            .get_mut(&c.a)
            .unwrap()
            .connections
            .retain(|c| *c != id);
        self.nodes
            .get_mut(&c.b)
            .unwrap()
            .connections
            .retain(|c| *c != id);
        true
    }
    pub fn remove_node(&mut self, id: NodeId) -> bool {
        let Some(n) = self.nodes.get(&id) else {
            return false;
        };
        for c in n.connections.clone() {
            self.disconnect(c);
        }
        self.nodes.remove(&id);
        true
    }
    pub fn components(&self) -> Vec<BTreeSet<NodeId>> {
        let mut unseen: BTreeSet<_> = self.nodes.keys().copied().collect();
        let mut components = vec![];
        while let Some(start) = unseen.pop_first() {
            let mut found = BTreeSet::from([start]);
            let mut queue = VecDeque::from([start]);
            while let Some(id) = queue.pop_front() {
                for c in &self.nodes[&id].connections {
                    let other = self.connections[c].other(id).unwrap();
                    if unseen.remove(&other) {
                        found.insert(other);
                        queue.push_back(other);
                    }
                }
            }
            components.push(found);
        }
        components
    }

    pub fn controller_state(&self, component: &BTreeSet<NodeId>) -> ControllerState {
        let controllers: BTreeSet<_> = component
            .iter()
            .copied()
            .filter(|id| self.nodes[id].spec.controller.is_some())
            .collect();
        let Some(start) = controllers.first().copied() else {
            return ControllerState::None;
        };
        let mut visited = BTreeSet::from([start]);
        let mut queue = VecDeque::from([start]);
        let mut min = self.nodes[&start].spec.controller.unwrap();
        let mut max = min;
        while let Some(node) = queue.pop_front() {
            let pos = self.nodes[&node].spec.controller.unwrap();
            for axis in 0..3 {
                min[axis] = min[axis].min(pos[axis]);
                max[axis] = max[axis].max(pos[axis]);
                if i64::from(max[axis]) - i64::from(min[axis]) >= 7 {
                    return ControllerState::Conflict;
                }
            }
            for edge in &self.nodes[&node].connections {
                let other = self.connections[edge].other(node).unwrap();
                if controllers.contains(&other) && visited.insert(other) {
                    queue.push_back(other);
                }
            }
        }
        if visited != controllers {
            return ControllerState::Conflict;
        }
        let positions: BTreeSet<_> = controllers
            .iter()
            .map(|id| self.nodes[id].spec.controller.unwrap())
            .collect();
        for pos in &positions {
            let mut crosses = 0;
            for axis in 0..3 {
                let mut a = *pos;
                let mut b = *pos;
                a[axis] += 1;
                b[axis] -= 1;
                if positions.contains(&a) && positions.contains(&b) {
                    crosses += 1;
                }
            }
            if crosses > 1 {
                return ControllerState::Conflict;
            }
        }
        ControllerState::Online
    }

    pub fn calculate(&mut self, mode: ChannelMode) -> Vec<GridReport> {
        self.components()
            .into_iter()
            .map(|component| self.calculate_component(component, mode))
            .collect()
    }

    fn calculate_component(
        &mut self,
        component: BTreeSet<NodeId>,
        mode: ChannelMode,
    ) -> GridReport {
        let state = self.controller_state(&component);
        let edges: BTreeSet<_> = component
            .iter()
            .flat_map(|n| self.nodes[n].connections.iter().copied())
            .collect();
        for node in &component {
            let n = self.nodes.get_mut(node).unwrap();
            n.used = 0;
            n.ancestor = None;
        }
        for edge in &edges {
            self.connections.get_mut(edge).unwrap().used = 0;
        }
        let (count, by_blocks, error) = match state {
            ControllerState::None => {
                let mut ignore = BTreeSet::new();
                let mut count = 0;
                let mut error = None;
                let cannot_compress = component
                    .iter()
                    .any(|n| self.nodes[n].spec.flags.has(Flags::CANNOT_CARRY_COMPRESSED));
                for id in &component {
                    let node = &self.nodes[id];
                    let flags = node.spec.flags;
                    if !flags.has(Flags::REQUIRE_CHANNEL) || ignore.contains(id) {
                        continue;
                    }
                    if flags.has(Flags::COMPRESSED_CHANNEL) && cannot_compress {
                        error = Some(AdHocError::NestedP2P);
                        break;
                    }
                    count += 1;
                    if flags.has(Flags::MULTIBLOCK) {
                        ignore.extend(node.spec.multiblock.iter().copied());
                    }
                }
                if error.is_none() && count > mode.ad_hoc_limit() {
                    error = Some(AdHocError::TooManyChannels);
                }
                if error.is_some() {
                    count = 0;
                }
                for id in &component {
                    self.nodes.get_mut(id).unwrap().used = count;
                }
                for id in &edges {
                    self.connections.get_mut(id).unwrap().used = count;
                }
                (count, component.len() as u64 * u64::from(count), error)
            }
            ControllerState::Conflict => (0, 0, None),
            ControllerState::Online => {
                let (count, by_blocks) = self.path_controllers(&component, mode);
                (count, by_blocks, None)
            }
        };
        let mut changed_nodes = vec![];
        let mut changed_connections = vec![];
        for id in &component {
            let node = self.nodes.get_mut(id).unwrap();
            node.ancestor = None;
            if !node.spec.flags.has(Flags::CANNOT_CARRY) && node.finalized != node.used {
                node.finalized = node.used;
                changed_nodes.push(*id);
            }
        }
        for id in &edges {
            let edge = self.connections.get_mut(id).unwrap();
            if edge.finalized != edge.used {
                edge.finalized = edge.used;
                changed_connections.push(*id);
            }
        }
        GridReport {
            nodes: component,
            controller: state,
            ad_hoc_error: error,
            channels_in_use: count,
            channels_by_blocks: by_blocks,
            changed_nodes,
            changed_connections,
        }
    }

    fn options(&self, item: PathItem) -> Vec<PathItem> {
        match item {
            PathItem::Node(id) => self.nodes[&id]
                .connections
                .iter()
                .map(|c| PathItem::Connection(*c))
                .collect(),
            PathItem::Connection(id) => {
                let c = &self.connections[&id];
                vec![PathItem::Node(c.a), PathItem::Node(c.b)]
            }
        }
    }
    fn route(&self, item: PathItem) -> Option<PathItem> {
        match item {
            PathItem::Node(id) => self.nodes[&id]
                .connections
                .first()
                .map(|c| PathItem::Connection(*c)),
            PathItem::Connection(id) => Some(PathItem::Node(self.connections[&id].a)),
        }
    }
    fn set_route(&mut self, item: PathItem, parent: PathItem, mode: ChannelMode) {
        match item {
            PathItem::Connection(id) => {
                if let PathItem::Node(node) = parent {
                    self.connections.get_mut(&id).unwrap().route_towards(node);
                }
            }
            PathItem::Node(id) => {
                let PathItem::Connection(edge) = parent else {
                    unreachable!()
                };
                let parent = self.connections[&edge].a;
                let p = &self.nodes[&parent];
                let node = &self.nodes[&id];
                let max = node.max_channels(mode);
                let (ancestor, subtree_max, subtree_compressed) = if p.spec.controller.is_some() {
                    (
                        None,
                        max,
                        !node.spec.flags.has(Flags::CANNOT_CARRY_COMPRESSED),
                    )
                } else {
                    let ancestor = match p.ancestor {
                        None => parent,
                        Some(ancestor) if p.subtree_max == self.nodes[&ancestor].subtree_max => {
                            ancestor
                        }
                        _ => parent,
                    };
                    (
                        Some(ancestor),
                        p.subtree_max.min(max),
                        p.subtree_compressed
                            && !node.spec.flags.has(Flags::CANNOT_CARRY_COMPRESSED),
                    )
                };
                let node = self.nodes.get_mut(&id).unwrap();
                node.used = 0;
                node.ancestor = ancestor;
                node.subtree_max = subtree_max;
                node.subtree_compressed = subtree_compressed;
                if let Some(index) = node.connections.iter().position(|c| *c == edge)
                    && index > 0
                {
                    node.connections.remove(index);
                    node.connections.insert(0, edge);
                }
            }
        }
    }
    fn enqueue(
        &self,
        item: PathItem,
        index: usize,
        queues: &mut [VecDeque<PathItem>; 3],
        visited: &mut BTreeSet<PathItem>,
    ) {
        visited.insert(item);
        let possible = match item {
            PathItem::Connection(_) => 0,
            PathItem::Node(id) => {
                let f = self.nodes[&id].spec.flags;
                if f.has(Flags::DENSE_CAPACITY) {
                    0
                } else if f.has(Flags::PREFERRED) {
                    1
                } else {
                    2
                }
            }
        };
        queues[index.max(possible)].push_back(item);
    }
    fn path_controllers(&mut self, component: &BTreeSet<NodeId>, mode: ChannelMode) -> (u32, u64) {
        let controllers: BTreeSet<_> = component
            .iter()
            .copied()
            .filter(|n| self.nodes[n].spec.controller.is_some())
            .collect();
        let mut visited: BTreeSet<_> = controllers.iter().map(|n| PathItem::Node(*n)).collect();
        let mut queues: [VecDeque<PathItem>; 3] = std::array::from_fn(|_| VecDeque::new());
        let mut roots = vec![];
        for node in &controllers {
            for edge in self.nodes[node].connections.clone() {
                let other = self.connections[&edge].other(*node).unwrap();
                if controllers.contains(&other) {
                    continue;
                }
                let item = PathItem::Connection(edge);
                self.enqueue(item, 0, &mut queues, &mut visited);
                self.set_route(item, PathItem::Node(*node), mode);
                roots.push(item);
            }
        }
        let mut bottlenecks: BTreeMap<NodeId, u32> = BTreeMap::new();
        let mut assigned = BTreeSet::new();
        let mut multiblocks = BTreeSet::new();
        for index in 0..3 {
            while let Some(item) = queues[index].pop_front() {
                for next in self.options(item) {
                    if visited.contains(&next) {
                        continue;
                    }
                    self.set_route(next, item, mode);
                    if let PathItem::Node(node) = next
                        && self.nodes[&node].spec.flags.has(Flags::REQUIRE_CHANNEL)
                        && !multiblocks.contains(&node)
                    {
                        let flags = self.nodes[&node].spec.flags;
                        let mut route = vec![];
                        let mut cursor = Some(node);
                        let mut valid = !(flags.has(Flags::COMPRESSED_CHANNEL)
                            && !self.nodes[&node].subtree_compressed);
                        while let Some(id) = cursor {
                            if bottlenecks.get(&id).copied().unwrap_or(0)
                                >= self.nodes[&id].max_channels(mode)
                            {
                                valid = false;
                                break;
                            }
                            route.push(id);
                            cursor = self.nodes[&id].ancestor;
                        }
                        if valid {
                            for id in route {
                                *bottlenecks.entry(id).or_default() += 1;
                            }
                            assigned.insert(node);
                            if flags.has(Flags::MULTIBLOCK) {
                                multiblocks.extend(
                                    self.nodes[&node]
                                        .spec
                                        .multiblock
                                        .iter()
                                        .copied()
                                        .filter(|n| *n != node),
                                );
                            }
                        }
                    }
                    self.enqueue(next, index, &mut queues, &mut visited);
                }
            }
        }
        // Iterative DFS, including only children of the BFS forest. No recursive
        // calls: a long cable chain must not overflow a thread's stack.
        let mut stack: Vec<_> = roots.into_iter().map(|p| (p, false)).collect();
        let mut count = 0;
        let mut by_blocks = 0u64;
        while let Some((item, finish)) = stack.pop() {
            if !finish {
                stack.push((item, true));
                for child in self.options(item) {
                    if matches!(child,PathItem::Node(n) if controllers.contains(&n)) {
                        continue;
                    }
                    if self.route(child) == Some(item) {
                        stack.push((child, false));
                    }
                }
            } else {
                match item {
                    PathItem::Connection(id) => {
                        let edge = &self.connections[&id];
                        let b = edge.b;
                        let used = if self.route(PathItem::Node(b)) == Some(item) {
                            self.nodes[&b].used
                        } else {
                            0
                        };
                        self.connections.get_mut(&id).unwrap().used = used;
                        by_blocks += u64::from(used);
                    }
                    PathItem::Node(id) => {
                        let mut used = u32::from(assigned.contains(&id));
                        for edge in &self.nodes[&id].connections {
                            if self.connections[edge].a == id {
                                used += self.connections[edge].used;
                            }
                        }
                        self.nodes.get_mut(&id).unwrap().used = used;
                        by_blocks += u64::from(used);
                        count += u32::from(assigned.contains(&id));
                    }
                }
            }
        }
        for id in multiblocks {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.used += 1;
            }
        }
        (count, by_blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn node(g: &mut Graph, flags: Flags) -> NodeId {
        g.add_node(NodeSpec {
            flags,
            ..Default::default()
        })
    }
    fn controller(g: &mut Graph, pos: Pos) -> NodeId {
        g.add_node(NodeSpec {
            controller: Some(pos),
            ..Default::default()
        })
    }
    fn link(g: &mut Graph, a: NodeId, b: NodeId) -> ConnectionId {
        g.connect(a, b, None).unwrap()
    }
    #[test]
    fn connection_invariants_direction_and_split_merge() {
        let mut g = Graph::default();
        let a = node(&mut g, Flags::default());
        let b = node(&mut g, Flags::default());
        assert!(g.connect(a, a, None).is_err());
        let edge = g.connect(a, b, Some(Direction::East)).unwrap();
        assert!(g.connect(b, a, None).is_err());
        assert_eq!(g.components().len(), 1);
        assert_eq!(
            g.connection(edge).unwrap().direction(b),
            Some(Direction::West)
        );
        assert!(g.disconnect(edge));
        assert_eq!(g.components().len(), 2);
        link(&mut g, a, b);
        assert_eq!(g.components().len(), 1);
        assert!(g.remove_node(a));
        assert_eq!(g.node(b).unwrap().connections().len(), 0);
    }
    #[test]
    fn ad_hoc_ninth_device_disables_every_consumer_and_removal_recovers() {
        let mut g = Graph::default();
        let cable = node(&mut g, Flags::PREFERRED);
        let mut devices = vec![];
        for _ in 0..8 {
            let d = node(&mut g, Flags::REQUIRE_CHANNEL);
            link(&mut g, cable, d);
            devices.push(d);
        }
        let r = g.calculate(ChannelMode::Default).remove(0);
        assert_eq!(r.channels_in_use, 8);
        assert_eq!(r.channels_by_blocks, 72);
        let ninth = node(&mut g, Flags::REQUIRE_CHANNEL);
        link(&mut g, cable, ninth);
        let r = g.calculate(ChannelMode::Default).remove(0);
        assert_eq!(r.ad_hoc_error, Some(AdHocError::TooManyChannels));
        assert!(
            devices
                .iter()
                .all(|n| !g.node(*n).unwrap().meets_channel_requirements())
        );
        g.remove_node(ninth);
        assert_eq!(g.calculate(ChannelMode::Default)[0].channels_in_use, 8);
    }
    #[test]
    fn ad_hoc_multiblock_uses_one_and_nested_compressed_is_rejected() {
        let mut g = Graph::default();
        let a = node(&mut g, Flags::REQUIRE_CHANNEL | Flags::MULTIBLOCK);
        let b = node(&mut g, Flags::REQUIRE_CHANNEL | Flags::MULTIBLOCK);
        g.node_mut(a).unwrap().spec.multiblock = BTreeSet::from([a, b]);
        g.node_mut(b).unwrap().spec.multiblock = BTreeSet::from([a, b]);
        link(&mut g, a, b);
        assert_eq!(g.calculate(ChannelMode::Default)[0].channels_in_use, 1);
        let inner = node(&mut g, Flags::REQUIRE_CHANNEL | Flags::COMPRESSED_CHANNEL);
        let outer = node(&mut g, Flags::CANNOT_CARRY_COMPRESSED);
        link(&mut g, a, inner);
        link(&mut g, a, outer);
        assert_eq!(
            g.calculate(ChannelMode::Default)[0].ad_hoc_error,
            Some(AdHocError::NestedP2P)
        );
    }
    #[test]
    fn controllers_require_direct_connectivity_maximum_seven_and_no_crosses() {
        let mut g = Graph::default();
        let a = controller(&mut g, [0, 0, 0]);
        let b = controller(&mut g, [6, 0, 0]);
        link(&mut g, a, b);
        assert_eq!(
            g.calculate(ChannelMode::Default)[0].controller,
            ControllerState::Online
        );
        g.node_mut(b).unwrap().spec.controller = Some([7, 0, 0]);
        assert_eq!(
            g.calculate(ChannelMode::Default)[0].controller,
            ControllerState::Conflict
        );
        g.node_mut(b).unwrap().spec.controller = Some([1, 0, 0]);
        let c = controller(&mut g, [-1, 0, 0]);
        link(&mut g, a, c);
        let d = controller(&mut g, [0, 1, 0]);
        link(&mut g, a, d);
        let e = controller(&mut g, [0, -1, 0]);
        link(&mut g, a, e);
        assert_eq!(
            g.calculate(ChannelMode::Default)[0].controller,
            ControllerState::Conflict
        );
        g.remove_node(e);
        assert_eq!(
            g.calculate(ChannelMode::Default)[0].controller,
            ControllerState::Online
        );
        let remote = controller(&mut g, [2, 0, 0]);
        let cable = node(&mut g, Flags::PREFERRED);
        link(&mut g, b, cable);
        link(&mut g, cable, remote);
        assert_eq!(
            g.calculate(ChannelMode::Default)[0].controller,
            ControllerState::Conflict
        );
    }
    #[test]
    fn normal_and_dense_bottlenecks_assign_channels_along_paths() {
        for (flags, limit) in [(Flags::PREFERRED, 8), (Flags::DENSE_CAPACITY, 32)] {
            let mut g = Graph::default();
            let root = controller(&mut g, [0, 0, 0]);
            let cable = node(&mut g, flags);
            let edge = link(&mut g, root, cable);
            let mut leaves = vec![];
            for _ in 0..40 {
                let d = node(&mut g, Flags::REQUIRE_CHANNEL);
                link(&mut g, cable, d);
                leaves.push(d);
            }
            let r = g.calculate(ChannelMode::Default).remove(0);
            assert_eq!(r.channels_in_use, limit);
            assert_eq!(g.node(cable).unwrap().used_channels(), limit);
            assert_eq!(g.connection(edge).unwrap().used_channels(), limit);
            assert_eq!(
                leaves
                    .iter()
                    .filter(|n| g.node(**n).unwrap().meets_channel_requirements())
                    .count(),
                limit as usize
            );
            assert_eq!(g.node(root).unwrap().used_channels(), 0);
            assert_eq!(r.channels_by_blocks, u64::from(limit) * 4);
        }
    }
    #[test]
    fn downstream_normal_cable_does_not_limit_an_independent_dense_branch() {
        let mut g = Graph::default();
        let root = controller(&mut g, [0, 0, 0]);
        let dense = node(&mut g, Flags::DENSE_CAPACITY);
        link(&mut g, root, dense);
        for flags in [Flags::PREFERRED, Flags::DENSE_CAPACITY] {
            let branch = node(&mut g, flags);
            link(&mut g, dense, branch);
            for _ in 0..20 {
                let d = node(&mut g, Flags::REQUIRE_CHANNEL);
                link(&mut g, branch, d);
            }
        }
        assert_eq!(g.calculate(ChannelMode::Default)[0].channels_in_use, 28);
        assert_eq!(g.node(dense).unwrap().used_channels(), 28);
    }
    #[test]
    fn compressed_channels_cannot_cross_outer_p2p_nodes() {
        let mut g = Graph::default();
        let root = controller(&mut g, [0, 0, 0]);
        let outer = node(
            &mut g,
            Flags::DENSE_CAPACITY | Flags::CANNOT_CARRY_COMPRESSED,
        );
        link(&mut g, root, outer);
        let inner = node(&mut g, Flags::REQUIRE_CHANNEL | Flags::COMPRESSED_CHANNEL);
        link(&mut g, outer, inner);
        let normal = node(&mut g, Flags::REQUIRE_CHANNEL);
        link(&mut g, outer, normal);
        assert_eq!(g.calculate(ChannelMode::Default)[0].channels_in_use, 1);
        assert!(!g.node(inner).unwrap().meets_channel_requirements());
        assert!(g.node(normal).unwrap().meets_channel_requirements());
    }
    #[test]
    fn dense_preferred_path_is_used_before_short_device_path() {
        let mut g = Graph::default();
        let root = controller(&mut g, [0, 0, 0]);
        let device = node(&mut g, Flags::REQUIRE_CHANNEL);
        link(&mut g, root, device);
        let dense = node(&mut g, Flags::DENSE_CAPACITY);
        let de = link(&mut g, root, dense);
        let preferred = node(&mut g, Flags::PREFERRED);
        link(&mut g, dense, preferred);
        let target = node(&mut g, Flags::REQUIRE_CHANNEL);
        link(&mut g, device, target);
        link(&mut g, preferred, target);
        assert_eq!(g.calculate(ChannelMode::Default)[0].channels_in_use, 2);
        assert_eq!(g.connection(de).unwrap().used_channels(), 1);
        assert_eq!(g.node(device).unwrap().used_channels(), 1);
    }
    #[test]
    fn all_channel_modes_scale_and_cannot_carry_stays_zero() {
        for mode in [
            ChannelMode::Default,
            ChannelMode::X2,
            ChannelMode::X3,
            ChannelMode::X4,
            ChannelMode::Infinite,
        ] {
            let mut g = Graph::default();
            let root = controller(&mut g, [0, 0, 0]);
            let c = node(&mut g, Flags::PREFERRED);
            link(&mut g, root, c);
            for _ in 0..40 {
                let d = node(&mut g, Flags::REQUIRE_CHANNEL);
                link(&mut g, c, d);
            }
            assert_eq!(
                g.calculate(mode)[0].channels_in_use,
                40.min(mode.ad_hoc_limit())
            );
            let forbidden = node(&mut g, Flags::CANNOT_CARRY | Flags::REQUIRE_CHANNEL);
            link(&mut g, root, forbidden);
            g.calculate(mode);
            assert_eq!(g.node(forbidden).unwrap().used_channels(), 0);
        }
    }
    #[test]
    fn repeated_calculation_only_emits_changed_counts() {
        let mut g = Graph::default();
        let a = node(&mut g, Flags::REQUIRE_CHANNEL);
        let b = node(&mut g, Flags::PREFERRED);
        link(&mut g, a, b);
        assert_eq!(g.calculate(ChannelMode::Default)[0].changed_nodes.len(), 2);
        let r = g.calculate(ChannelMode::Default).remove(0);
        assert!(r.changed_nodes.is_empty());
        assert!(r.changed_connections.is_empty());
    }
    #[test]
    fn long_network_uses_iterative_bfs_and_dfs() {
        let mut g = Graph::default();
        let root = controller(&mut g, [0, 0, 0]);
        let mut prev = root;
        for _ in 0..10000 {
            let next = node(&mut g, Flags::PREFERRED);
            link(&mut g, prev, next);
            prev = next;
        }
        let last = node(&mut g, Flags::REQUIRE_CHANNEL);
        link(&mut g, prev, last);
        let r = g.calculate(ChannelMode::Default).remove(0);
        assert_eq!(r.channels_in_use, 1);
        assert_eq!(r.channels_by_blocks, 20002);
    }
}
