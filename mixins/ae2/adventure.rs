// SPDX-License-Identifier: LGPL-3.0-or-later
// AdventureModePredicate/BlockPredicate saved form and property/NBT matching.
// Engine adapters supply the actual registry, typed domains and block-entity data.
use std::collections::BTreeMap;

pub trait Value: Clone {
    type Key: Eq;
    fn kind(&self) -> u8;
    fn text(&self) -> Option<String>;
    fn boolean(&self) -> Option<bool>;
    fn field(&self, name: &str) -> Option<&Self>;
    fn entry(&self, key: &Self::Key) -> Option<&Self>;
    fn entries(&self) -> Option<Vec<(&Self::Key, &Self)>>;
    fn key_text(key: &Self::Key) -> String;
    fn stream(&self) -> Option<Vec<Self>>;
    fn list(&self) -> Option<&[Self]>;
    fn scalar_equal(&self, other: &Self) -> bool;
}
pub trait Registry<T: Value> {
    fn block_exists(&self, id: &str) -> bool;
    fn tag_exists(&self, id: &str) -> bool;
    fn compound(&self, value: &T) -> Option<T>;
}
pub trait World<T: Value> {
    fn block(&self) -> Option<&str>;
    fn property_rank(&self, name: &str, value: &str) -> Option<i32>;
    fn current_rank(&self, name: &str) -> Option<i32>;
    fn in_tag(&self, tag: &str) -> bool;
    fn entity(&self) -> Option<T>;
}
#[derive(Clone, Debug)]
pub enum Blocks {
    Direct(Vec<String>),
    Tag(String),
}
#[derive(Clone, Debug)]
pub enum Matcher {
    Exact(String),
    Range {
        min: Option<String>,
        max: Option<String>,
    },
}
#[derive(Clone, Debug)]
pub struct Block<T> {
    pub blocks: Option<Blocks>,
    pub state: BTreeMap<String, Matcher>,
    pub nbt: Option<T>,
}
#[derive(Clone, Debug)]
pub struct Predicate<T> {
    pub predicates: Vec<Block<T>>,
    pub show_in_tooltip: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
fn resource(value: String) -> Result<String, Invalid> {
    let (namespace, path) = value.split_once(':').unwrap_or(("minecraft", &value));
    let namespace = if namespace.is_empty() {
        "minecraft"
    } else {
        namespace
    };
    let valid_ns = namespace
        .bytes()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_.-".contains(&c));
    let valid_path = path
        .bytes()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_.-/".contains(&c));
    if !valid_ns || !valid_path {
        return Err(Invalid);
    }
    Ok(format!("{namespace}:{path}"))
}
fn direct<T: Value>(value: &T, registry: &impl Registry<T>) -> Result<String, Invalid> {
    let id = resource(value.text().ok_or(Invalid)?)?;
    if !registry.block_exists(&id) {
        return Err(Invalid);
    }
    Ok(id)
}
fn block<T: Value>(value: &T, registry: &impl Registry<T>) -> Result<Block<T>, Invalid> {
    if value.entries().is_none() {
        return Err(Invalid);
    }
    let blocks = if let Some(v) = value.field("blocks") {
        Some(if let Some(text) = v.text() {
            if let Some(tag) = text.strip_prefix('#') {
                let id = resource(tag.to_owned())?;
                if !registry.tag_exists(&id) {
                    return Err(Invalid);
                }
                Blocks::Tag(id)
            } else {
                Blocks::Direct(vec![direct(v, registry)?])
            }
        } else {
            Blocks::Direct(
                v.stream()
                    .ok_or(Invalid)?
                    .iter()
                    .map(|v| direct(v, registry))
                    .collect::<Result<_, _>>()?,
            )
        })
    } else {
        None
    };
    let mut state = BTreeMap::new();
    if let Some(v) = value.field("state") {
        for (key, value) in v.entries().ok_or(Invalid)? {
            let matcher = if let Some(text) = value.text() {
                Matcher::Exact(text)
            } else {
                value.entries().ok_or(Invalid)?;
                let bound = |name| {
                    value
                        .field(name)
                        .map(|v| v.text().ok_or(Invalid))
                        .transpose()
                };
                Matcher::Range {
                    min: bound("min")?,
                    max: bound("max")?,
                }
            };
            state.insert(T::key_text(key), matcher);
        }
    }
    let nbt = value
        .field("nbt")
        .map(|v| registry.compound(v).ok_or(Invalid))
        .transpose()?;
    Ok(Block { blocks, state, nbt })
}
pub fn decode<T: Value>(value: &T, registry: &impl Registry<T>) -> Result<Predicate<T>, Invalid> {
    // Codec.withAlternative(FULL_CODEC, SIMPLE_CODEC), not a flatten/guess of
    // an arbitrary compound. A failed full form can legally fall back to a
    // simple BlockPredicate whose unknown fields the original codec ignores.
    let full = || {
        value.entries().ok_or(Invalid)?;
        let list = value
            .field("predicates")
            .and_then(Value::stream)
            .ok_or(Invalid)?;
        if list.is_empty() {
            return Err(Invalid);
        }
        let predicates = list
            .iter()
            .map(|v| block(v, registry))
            .collect::<Result<_, _>>()?;
        let show_in_tooltip = value
            .field("show_in_tooltip")
            .map(|v| v.boolean().ok_or(Invalid))
            .transpose()?
            .unwrap_or(true);
        Ok(Predicate {
            predicates,
            show_in_tooltip,
        })
    };
    full().or_else(|_| {
        Ok(Predicate {
            predicates: vec![block(value, registry)?],
            show_in_tooltip: true,
        })
    })
}
impl<T: Value> Predicate<T> {
    pub fn matches_fresh(&self, world: &impl World<T>) -> bool {
        self.predicates
            .iter()
            .any(|predicate| predicate.matches(world))
    }
}
impl<T: Value> Block<T> {
    pub fn matches(&self, world: &impl World<T>) -> bool {
        if let Some(blocks) = &self.blocks {
            let accepted = match blocks {
                Blocks::Direct(ids) => world
                    .block()
                    .is_some_and(|id| ids.iter().any(|want| want == id)),
                Blocks::Tag(id) => world.in_tag(id),
            };
            if !accepted {
                return false;
            }
        }
        for (name, matcher) in &self.state {
            let Some(current) = world.current_rank(name) else {
                return false;
            };
            match matcher {
                Matcher::Exact(text) => {
                    if world.property_rank(name, text) != Some(current) {
                        return false;
                    }
                }
                Matcher::Range { min, max } => {
                    if min.as_ref().is_some_and(|text| {
                        world
                            .property_rank(name, text)
                            .is_none_or(|bound| current < bound)
                    }) || max.as_ref().is_some_and(|text| {
                        world
                            .property_rank(name, text)
                            .is_none_or(|bound| current > bound)
                    }) {
                        return false;
                    }
                }
            }
        }
        self.nbt.as_ref().is_none_or(|expected| {
            world
                .entity()
                .as_ref()
                .is_some_and(|actual| compare_nbt(expected, actual))
        })
    }
}
/// NbtUtils.compareNbt(expected, actual, true): list entries are existential,
/// including duplicates, after the source's minimum list-length check.
pub fn compare_nbt<T: Value>(expected: &T, actual: &T) -> bool {
    if expected.kind() != actual.kind() {
        return false;
    }
    if let Some(entries) = expected.entries() {
        entries
            .iter()
            .all(|(key, value)| actual.entry(key).is_some_and(|a| compare_nbt(*value, a)))
    } else if let Some(want) = expected.list() {
        let Some(have) = actual.list() else {
            return false;
        };
        if want.is_empty() {
            have.is_empty()
        } else {
            have.len() >= want.len() && want.iter().all(|w| have.iter().any(|h| compare_nbt(w, h)))
        }
    } else {
        expected.scalar_equal(actual)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Boolean,
    Integer,
    Enum,
}
#[derive(Clone, Debug)]
pub struct Domain {
    pub kind: Kind,
    pub values: BTreeMap<String, i32>,
}
impl Domain {
    pub fn rank(&self, value: &str) -> Option<i32> {
        if self.kind == Kind::Integer {
            let value = java_integer(value)?;
            self.values.get(&value.to_string()).copied()
        } else {
            self.values.get(value).copied()
        }
    }
}
/// Integer.valueOf used by IntegerProperty, including BMP decimal digits.
pub fn java_integer(value: &str) -> Option<i32> {
    let (negative, digits) = if let Some(v) = value.strip_prefix('-') {
        (true, v)
    } else {
        (false, value.strip_prefix('+').unwrap_or(value))
    };
    if digits.is_empty() {
        return None;
    }
    let mut result = 0i64;
    for ch in digits.chars() {
        let code = ch as u32;
        let digit = DIGITS
            .iter()
            .find_map(|&zero| (code >= zero && code < zero + 10).then(|| code - zero))?;
        result = result.checked_mul(10)?.checked_add(i64::from(digit))?;
        if result > if negative { 2147483648 } else { 2147483647 } {
            return None;
        }
    }
    Some(if negative { -result } else { result } as i32)
}
// Character.digit(char, 10), Java 21 / Unicode 15. Supplementary code points
// are not accepted by Integer.parseInt's char-by-char loop.
const DIGITS: &[u32] = &[
    0x0030, 0x0660, 0x06f0, 0x07c0, 0x0966, 0x09e6, 0x0a66, 0x0ae6, 0x0b66, 0x0be6, 0x0c66, 0x0ce6,
    0x0d66, 0x0de6, 0x0e50, 0x0ed0, 0x0f20, 0x1040, 0x1090, 0x17e0, 0x1810, 0x1946, 0x19d0, 0x1a80,
    0x1a90, 0x1b50, 0x1bb0, 0x1c40, 0x1c50, 0xa620, 0xa8d0, 0xa900, 0xa9d0, 0xa9f0, 0xaa50, 0xabf0,
    0xff10,
];
