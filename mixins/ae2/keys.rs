// SPDX-License-Identifier: LGPL-3.0-or-later
// AEKey/AEItemKey/AEFluidKey, GenericStack, KeyCounter and FuzzySearch rules.
// Copyright AE2 contributors; 2026 Horizon port contributors. See NOTICE.md.
// Component bytes are canonical, typed NBT supplied by the native adapter.
// Neither stack count nor runtime/registry metadata participates in key identity.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyType {
    Item,
    Fluid,
}

impl KeyType {
    pub fn amount_per_byte(self) -> u64 {
        match self {
            Self::Item => 8,
            Self::Fluid => 8000,
        }
    }
    pub fn total_types(self) -> usize {
        match self {
            Self::Item => 63,
            Self::Fluid => 18,
        }
    }
    pub fn codec_id(self) -> &'static str {
        match self {
            Self::Item => "ae2:i",
            Self::Fluid => "ae2:f",
        }
    }
    pub fn from_codec_id(id: &str) -> Option<Self> {
        match id {
            "ae2:i" => Some(Self::Item),
            "ae2:f" => Some(Self::Fluid),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FuzzyMode {
    #[default]
    IgnoreAll,
    Percent99,
    Percent75,
    Percent50,
    Percent25,
}

impl FuzzyMode {
    pub const ALL: [Self; 5] = [
        Self::IgnoreAll,
        Self::Percent99,
        Self::Percent75,
        Self::Percent50,
        Self::Percent25,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::IgnoreAll => "IGNORE_ALL",
            Self::Percent99 => "PERCENT_99",
            Self::Percent75 => "PERCENT_75",
            Self::Percent50 => "PERCENT_50",
            Self::Percent25 => "PERCENT_25",
        }
    }
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.name() == name)
    }
    fn percentage(self) -> f32 {
        match self {
            Self::IgnoreAll => -1.0,
            Self::Percent99 => 0.0,
            Self::Percent75 => 25.0,
            Self::Percent50 => 50.0,
            Self::Percent25 => 75.0,
        }
    }
    pub fn breakpoint(self, max_damage: i32) -> i32 {
        // Preserve Java's float rounding and truncation, not integer percentages.
        (self.percentage() * max_damage as f32 / 100.0) as i32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyProperties {
    pub damage: i32,
    pub max_damage: i32,
    pub max_stack: u32,
    pub can_fit_inside_cell: bool,
    pub blacklisted: bool,
}

impl Default for KeyProperties {
    fn default() -> Self {
        Self {
            damage: 0,
            max_damage: 0,
            max_stack: 64,
            can_fit_inside_cell: true,
            blacklisted: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Key {
    kind: KeyType,
    id: String,
    components: Vec<u8>,
    properties: KeyProperties,
}

impl Key {
    pub fn new(
        kind: KeyType,
        id: String,
        components: Vec<u8>,
        properties: KeyProperties,
    ) -> Option<Self> {
        let (namespace, path) = id.split_once(':')?;
        if namespace.is_empty()
            || path.is_empty()
            || !namespace
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_.-".contains(&c))
            || !path
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"/_.-".contains(&c))
            || kind == KeyType::Item && id == "minecraft:air"
            || kind == KeyType::Fluid && id == "minecraft:empty"
        {
            return None;
        }
        Some(Self {
            kind,
            id,
            components,
            properties,
        })
    }
    pub fn kind(&self) -> KeyType {
        self.kind
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn components(&self) -> &[u8] {
        &self.components
    }
    pub fn properties(&self) -> KeyProperties {
        self.properties
    }
    pub fn same_primary(&self, other: &Self) -> bool {
        self.kind == other.kind && self.id == other.id
    }

    /// AEKey.fuzzyEquals: item types support ranges even when max damage is 0.
    pub fn fuzzy_equals(&self, other: &Self, mode: FuzzyMode) -> bool {
        if !self.same_primary(other) {
            return false;
        }
        if self.kind == KeyType::Fluid || mode == FuzzyMode::IgnoreAll {
            return true;
        }
        if mode == FuzzyMode::Percent99 {
            return (self.properties.damage > 0) == (other.properties.damage > 0);
        }
        let a = self.properties.damage as f32 / self.properties.max_damage as f32;
        let b = other.properties.damage as f32 / other.properties.max_damage as f32;
        let point = mode.percentage() / 100.0;
        (a > point) == (b > point)
    }

    /// FuzzySearch's [lower, upper) descending damage interval, used by the
    /// variant map/partitions. It intentionally excludes invalid over-damage.
    pub fn fuzzy_search_matches(&self, candidate: &Self, mode: FuzzyMode) -> bool {
        if !self.same_primary(candidate) {
            return false;
        }
        let maximum = self.properties.max_damage;
        if maximum <= 0 {
            return true;
        } // UnorderedVariantMap
        let (lower, upper) = if mode == FuzzyMode::IgnoreAll {
            (maximum, -1)
        } else {
            let point = mode.breakpoint(maximum);
            if self.properties.damage <= point {
                (point, -1)
            } else {
                (maximum, point)
            }
        };
        candidate.properties.damage <= lower && candidate.properties.damage > upper
    }

    fn identity(&self) -> (KeyType, &str, &[u8]) {
        (self.kind, &self.id, &self.components)
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}
impl Eq for Key {}
impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        self.identity().cmp(&other.identity())
    }
}
impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity().hash(state);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericStack {
    pub what: Key,
    pub amount: i64,
}

/// Signed quantities are required for crafting deficits. Zero entries remain
/// until remove_zeros, matching KeyCounter (do not confuse this with inventory).
#[derive(Clone, Debug, Default)]
pub struct KeyCounter {
    entries: BTreeMap<Key, i64>,
}

impl KeyCounter {
    pub fn get(&self, key: &Key) -> i64 {
        self.entries.get(key).copied().unwrap_or(0)
    }
    pub fn set(&mut self, key: Key, amount: i64) {
        self.entries.insert(key, amount);
    }
    pub fn add(&mut self, key: Key, amount: i64) {
        let next = self.get(&key).wrapping_add(amount);
        self.set(key, next);
    }
    pub fn remove(&mut self, key: Key, amount: i64) {
        self.add(key, amount.wrapping_neg());
    }
    pub fn remove_key(&mut self, key: &Key) -> i64 {
        self.entries.remove(key).unwrap_or(0)
    }
    pub fn add_all(&mut self, other: &Self) {
        for (k, v) in &other.entries {
            self.add(k.clone(), *v);
        }
    }
    pub fn remove_all(&mut self, other: &Self) {
        for (k, v) in &other.entries {
            self.remove(k.clone(), *v);
        }
    }
    pub fn reset(&mut self) {
        self.entries.values_mut().for_each(|v| *v = 0);
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn remove_zeros(&mut self) {
        self.entries.retain(|_, v| *v != 0);
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&Key, i64)> {
        self.entries.iter().map(|(k, v)| (k, *v))
    }
    pub fn find_fuzzy<'a>(
        &'a self,
        filter: &'a Key,
        mode: FuzzyMode,
    ) -> impl Iterator<Item = (&'a Key, i64)> {
        self.iter()
            .filter(move |(key, _)| filter.fuzzy_search_matches(key, mode))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(damage: i32, maximum: i32, components: &[u8]) -> Key {
        Key::new(
            KeyType::Item,
            "minecraft:diamond_pickaxe".into(),
            components.to_vec(),
            KeyProperties {
                damage,
                max_damage: maximum,
                ..Default::default()
            },
        )
        .unwrap()
    }
    #[test]
    fn identity_is_component_sensitive_and_metadata_independent() {
        assert_eq!(item(1, 100, &[1]), item(2, 100, &[1]));
        assert_ne!(item(1, 100, &[1]), item(1, 100, &[2]));
        let mut counter = KeyCounter::default();
        counter.add(item(1, 100, &[1]), 10);
        counter.add(item(2, 100, &[1]), 20);
        assert_eq!(counter.len(), 1);
        assert_eq!(counter.get(&item(0, 0, &[1])), 30);
    }
    #[test]
    fn invalid_ids_and_empty_resources_are_rejected() {
        for id in [
            "air",
            "minecraft:air",
            "Minecraft:stone",
            "a:",
            ":a",
            "a:has space",
        ] {
            assert!(Key::new(KeyType::Item, id.into(), vec![], Default::default()).is_none());
        }
        assert!(
            Key::new(
                KeyType::Fluid,
                "minecraft:empty".into(),
                vec![],
                Default::default()
            )
            .is_none()
        );
    }
    #[test]
    fn damage_boundaries_follow_java_fuzzy_search_not_percent_labels() {
        for (mode, point) in [
            (FuzzyMode::Percent99, 0),
            (FuzzyMode::Percent75, 25),
            (FuzzyMode::Percent50, 50),
            (FuzzyMode::Percent25, 75),
        ] {
            let low = item(point, 100, &[1]);
            let high = item(point + 1, 100, &[2]);
            assert!(low.fuzzy_search_matches(&item(0, 100, &[3]), mode));
            assert!(!low.fuzzy_search_matches(&high, mode));
            assert!(high.fuzzy_search_matches(&item(100, 100, &[4]), mode));
            assert!(!high.fuzzy_search_matches(&low, mode));
        }
    }
    #[test]
    fn ignore_all_search_excludes_negative_and_over_max_damage() {
        let key = item(0, 100, &[]);
        assert!(!key.fuzzy_search_matches(&item(-1, 100, &[]), FuzzyMode::IgnoreAll));
        assert!(!key.fuzzy_search_matches(&item(101, 100, &[]), FuzzyMode::IgnoreAll));
        assert!(key.fuzzy_equals(&item(101, 100, &[]), FuzzyMode::IgnoreAll));
    }
    #[test]
    fn nondamageable_and_fluid_search_ignores_secondary_components() {
        assert!(item(0, 0, &[1]).fuzzy_search_matches(&item(0, 0, &[2]), FuzzyMode::Percent50));
        let a = Key::new(
            KeyType::Fluid,
            "minecraft:water".into(),
            vec![1],
            Default::default(),
        )
        .unwrap();
        let b = Key::new(
            KeyType::Fluid,
            "minecraft:water".into(),
            vec![2],
            Default::default(),
        )
        .unwrap();
        assert!(a.fuzzy_search_matches(&b, FuzzyMode::Percent99));
        assert!(!a.same_primary(&item(0, 0, &[1])));
    }
    #[test]
    fn counters_preserve_deficits_zeros_and_java_long_wrap() {
        let mut a = KeyCounter::default();
        let key = item(0, 0, &[]);
        a.remove(key.clone(), 5);
        assert_eq!(a.get(&key), -5);
        a.add(key.clone(), 5);
        assert_eq!(a.len(), 1);
        assert!(!a.is_empty());
        a.remove_zeros();
        assert!(a.is_empty());
        a.set(key.clone(), i64::MAX);
        a.add(key.clone(), 1);
        assert_eq!(a.get(&key), i64::MIN);
        a.reset();
        assert_eq!(a.get(&key), 0);
        assert_eq!(a.len(), 1);
        let mut b = KeyCounter::default();
        b.set(key.clone(), 42);
        a.remove_all(&b);
        assert_eq!(a.get(&key), -42);
        a.add_all(&b);
        a.remove_zeros();
        assert!(a.is_empty());
    }
    #[test]
    fn counter_fuzzy_search_keeps_distinct_component_variants() {
        let mut c = KeyCounter::default();
        for d in [0, 25, 26, 100, 101] {
            c.add(item(d, 100, &[d as u8]), 1);
        }
        assert_eq!(
            c.find_fuzzy(&item(0, 100, &[]), FuzzyMode::Percent75)
                .count(),
            2
        );
        assert_eq!(
            c.find_fuzzy(&item(26, 100, &[]), FuzzyMode::Percent75)
                .count(),
            2
        );
        assert_eq!(
            c.find_fuzzy(&item(0, 100, &[]), FuzzyMode::IgnoreAll)
                .count(),
            4
        );
    }
}
