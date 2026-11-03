// SPDX-License-Identifier: LGPL-3.0-or-later
// Original ItemStack adventure gate, evaluated on the actual acting stack.
use super::{
    Game,
    ae2_adventure::{self as rules, Domain, Kind, Registry, Value, World},
};
use horizon_client::{items::ItemStack, save};
use horizon_world::nbt::{JavaString, Tag};
use serde_json::Value as Json;
use std::cell::RefCell;

impl Value for Tag {
    type Key = JavaString;
    fn kind(&self) -> u8 {
        self.id()
    }
    fn text(&self) -> Option<String> {
        if let Self::String(s) = self {
            Some(s.to_string_lossy())
        } else {
            None
        }
    }
    fn boolean(&self) -> Option<bool> {
        match self {
            Self::Byte(v) => Some(*v != 0),
            Self::Short(v) => Some(*v as i8 != 0),
            Self::Int(v) => Some(*v as i8 != 0),
            Self::Long(v) => Some(*v as i8 != 0),
            Self::Float(v) => Some(f32::from_bits(*v) as i32 as i8 != 0),
            Self::Double(v) => Some(f64::from_bits(*v) as i32 as i8 != 0),
            _ => None,
        }
    }
    fn field(&self, name: &str) -> Option<&Self> {
        self.get(name)
    }
    fn entry(&self, key: &JavaString) -> Option<&Self> {
        if let Self::Compound(c) = self {
            c.get(key)
        } else {
            None
        }
    }
    fn entries(&self) -> Option<Vec<(&JavaString, &Self)>> {
        if let Self::Compound(c) = self {
            Some(c.iter().collect())
        } else {
            None
        }
    }
    fn key_text(key: &JavaString) -> String {
        key.to_string_lossy()
    }
    fn stream(&self) -> Option<Vec<Self>> {
        match self {
            Self::List(v) => Some(v.clone()),
            Self::ByteArray(v) => Some(v.iter().map(|&v| Self::Byte(v as i8)).collect()),
            Self::IntArray(v) => Some(v.iter().copied().map(Self::Int).collect()),
            Self::LongArray(v) => Some(v.iter().copied().map(Self::Long).collect()),
            _ => None,
        }
    }
    fn list(&self) -> Option<&[Self]> {
        if let Self::List(v) = self {
            Some(v)
        } else {
            None
        }
    }
    fn scalar_equal(&self, other: &Self) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }
        match (self, other) {
            (Self::Float(a), Self::Float(b)) => f32::from_bits(*a) == f32::from_bits(*b),
            (Self::Double(a), Self::Double(b)) => f64::from_bits(*a) == f64::from_bits(*b),
            _ => self == other,
        }
    }
}
impl super::ae2_adventure_snbt::Target for Tag {
    fn number(kind: u8, bits: u64) -> Self {
        match kind {
            1 => Self::Byte(bits as i8),
            2 => Self::Short(bits as i16),
            3 => Self::Int(bits as i32),
            4 => Self::Long(bits as i64),
            5 => Self::Float(bits as u32),
            6 => Self::Double(bits),
            _ => unreachable!(),
        }
    }
    fn string(v: Vec<u16>) -> Self {
        Self::String(JavaString(v))
    }
    fn compound(v: Vec<(Vec<u16>, Self)>) -> Self {
        Self::Compound(v.into_iter().map(|(k, v)| (JavaString(k), v)).collect())
    }
    fn sequence(kind: u8, v: Vec<Self>) -> Self {
        match kind {
            7 => Self::ByteArray(
                v.into_iter()
                    .map(|v| {
                        if let Self::Byte(n) = v {
                            n as u8
                        } else {
                            unreachable!()
                        }
                    })
                    .collect(),
            ),
            11 => Self::IntArray(
                v.into_iter()
                    .map(|v| {
                        if let Self::Int(n) = v {
                            n
                        } else {
                            unreachable!()
                        }
                    })
                    .collect(),
            ),
            12 => Self::LongArray(
                v.into_iter()
                    .map(|v| {
                        if let Self::Long(n) = v {
                            n
                        } else {
                            unreachable!()
                        }
                    })
                    .collect(),
            ),
            9 => Self::List(v),
            _ => unreachable!(),
        }
    }
}
struct At<'a, 'g> {
    game: RefCell<&'a mut Game<'g>>,
    pos: [i32; 3],
    block: String,
    domains: Json,
}
impl At<'_, '_> {
    fn domain(&self, name: &str) -> Option<Domain> {
        let d = self.domains.get("blocks")?.get(&self.block)?.get(name)?;
        let kind = match d["kind"].as_str()? {
            "integer" => Kind::Integer,
            "boolean" => Kind::Boolean,
            "enum" => Kind::Enum,
            _ => return None,
        };
        let values = d["values"]
            .as_object()?
            .iter()
            .map(|(k, v)| Some((k.clone(), i32::try_from(v.as_i64()?).ok()?)))
            .collect::<Option<_>>()?;
        Some(Domain { kind, values })
    }
}
impl Registry<Tag> for At<'_, '_> {
    fn block_exists(&self, id: &str) -> bool {
        self.game
            .borrow()
            .pack
            .assets
            .registry
            .default_state(id)
            .is_some()
    }
    fn tag_exists(&self, id: &str) -> bool {
        self.game.borrow().items().block_tags.contains_key(id)
    }
    fn compound(&self, value: &Tag) -> Option<Tag> {
        match value {
            Tag::Compound(_) => Some(value.clone()),
            Tag::String(text) => super::ae2_adventure_snbt::parse(&text.0),
            _ => None,
        }
    }
}
impl World<Tag> for At<'_, '_> {
    fn block(&self) -> Option<&str> {
        Some(&self.block)
    }
    fn property_rank(&self, name: &str, value: &str) -> Option<i32> {
        self.domain(name)?.rank(value)
    }
    fn current_rank(&self, name: &str) -> Option<i32> {
        self.domain(name)?
            .rank(&self.game.borrow().native_block_property(self.pos, name)?)
    }
    fn in_tag(&self, tag: &str) -> bool {
        self.game
            .borrow()
            .items()
            .block_tags
            .get(tag)
            .is_some_and(|ids| ids.contains(&self.block))
    }
    fn entity(&self) -> Option<Tag> {
        let p = (self.pos[0], self.pos[1], self.pos[2]);
        let mut game = self.game.borrow_mut();
        // Client Chunk.getBlockEntity creates the registered entity on demand.
        // Horizon's remote map initially contains only its renderable vanilla
        // entities and the separately streamed CableBus. Use the mod's real
        // constructor; never substitute a fabricated NBT mask or server save.
        if game.remote.is_some()
            && game.world.loaded(p.0, p.2)
            && !game.world.block_entities.contains_key(&p)
            && let Some(entity) = game.new_block_entity(&self.block)
        {
            game.world.block_entities.insert(p, entity);
        }
        Some(save::encode_block_entity(
            p,
            game.world.block_entities.get(&p)?,
            &self.block,
        ))
    }
}
pub(super) fn may_use(game: &mut Game<'_>, stack: &ItemStack, pos: [i32; 3]) -> bool {
    if game.player.is_none() || game.may_build() {
        return true;
    }
    let Some(value) = stack.data().and_then(|d| d.other("minecraft:can_place_on")) else {
        return false;
    };
    // This file belongs to AE2's content mod. Installed base resources are read-only.
    let domains = game
        .native_data("data/ae2/source-property-domains.json")
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or(Json::Null);
    let block = game.native_block(pos);
    let at = At {
        game: RefCell::new(game),
        pos,
        block,
        domains,
    };
    rules::decode(value, &at).is_ok_and(|predicate| predicate.matches_fresh(&at))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(s: &str) -> Option<Tag> {
        super::super::ae2_adventure_snbt::parse(&s.encode_utf16().collect::<Vec<_>>())
    }
    #[test]
    fn ae2_saved_nbt_uses_original_quoted_escapes_and_typed_sequences() {
        assert!(parse("{x:'\\n'}").is_none());
        assert!(parse("{x:1} trailing").is_none());
        assert!(parse("{x:[1,2b]}").is_none());
        assert!(parse("{x:[B;1]}").is_none());
        assert_eq!(
            parse("{x:[B;1b,-1b,]}"),
            Some(Tag::compound([("x", Tag::ByteArray(vec![1, 255]))]))
        );
        assert_eq!(
            parse("{x:[I;1,-2]}"),
            Some(Tag::compound([("x", Tag::IntArray(vec![1, -2]))]))
        );
        assert_eq!(
            parse("{x:[L;1l]}"),
            Some(Tag::compound([("x", Tag::LongArray(vec![1]))]))
        );
    }
    #[test]
    fn ae2_nbt_float_comparison_and_boolean_codec_use_java_numeric_rules() {
        assert!(Tag::Float(0).scalar_equal(&Tag::Float((-0.0f32).to_bits())));
        assert!(!Tag::Float(f32::NAN.to_bits()).scalar_equal(&Tag::Float(f32::NAN.to_bits())));
        assert_eq!(Tag::Int(256).boolean(), Some(false));
        assert_eq!(Tag::Double(257.9f64.to_bits()).boolean(), Some(true));
    }
}
