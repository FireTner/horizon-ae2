// SPDX-License-Identifier: LGPL-3.0-or-later
// Native transport only. Saved source cable/gn data is never rewritten here.
use crate::containers::ModBlockEntity;
use horizon_world::nbt::Tag;
const FIELD: &str = "ae2:cable_visual_v1";
pub fn encode(be: &ModBlockEntity) -> Tag {
    let mut tag = horizon_world::nbt::Compound::new();
    if let Some(part) = be.nbt.get(&"cable".into()) {
        tag.insert("cable".into(), part.clone());
    }
    tag.insert(FIELD.into(), Tag::IntArray(be.data.clone()));
    Tag::Compound(tag)
}
pub fn decode(tag: &Tag) -> Option<ModBlockEntity> {
    let Tag::Compound(fields) = tag else {
        return None;
    };
    if fields.is_empty() {
        return None;
    }
    let mut clean = fields.clone();
    let visual = clean.remove(&FIELD.into());
    clean.insert("id".into(), Tag::String("ae2:cable_bus".into()));
    for key in ["x", "y", "z"] {
        clean.insert(key.into(), Tag::Int(0));
    }
    let (_, crate::containers::BlockEntity::Mod(mut be)) =
        crate::save::decode_block_entity(&Tag::Compound(clean))?
    else {
        return None;
    };
    restore(&mut be, visual.as_ref());
    Some(*be)
}
pub fn restore(be: &mut ModBlockEntity, visual: Option<&Tag>) {
    if let Some(Tag::IntArray(data)) = visual
        && data.len() == 8
        && data[0] == 0x414532
        && (0..=63).contains(&data[1])
        && data[2..].iter().all(|n| (0..=32).contains(n))
    {
        be.data.clone_from(data);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ae2_cable_wire_roundtrip_keeps_visuals_out_of_saved_nbt() {
        let mut be = ModBlockEntity {
            id: "ae2:cable_bus".into(),
            nbt: Default::default(),
            items: vec![],
            data: vec![0x414532, 33, 0, 8, 0, 32, 0, 1],
        };
        be.nbt.insert(
            "cable".into(),
            Tag::compound([
                ("id", Tag::String("ae2:fluix_smart_cable".into())),
                ("gn", Tag::compound([("p", Tag::Int(7))])),
            ]),
        );
        be.nbt.insert("private".into(), Tag::Int(9));
        let original = be.nbt.clone();
        let Tag::Compound(mut wire) = encode(&be) else {
            panic!()
        };
        assert!(!wire.contains_key(&"private".into()));
        for (key, n) in [("x", 1), ("y", 70), ("z", -2)] {
            wire.insert(key.into(), Tag::Int(n));
        }
        wire.insert("id".into(), Tag::String(be.id.as_str().into()));
        let decoded = decode(&Tag::Compound(wire.clone())).unwrap();
        // Save decoding must preserve arbitrary source fields, including a
        // name also reserved by the native wire protocol.
        let (_, crate::containers::BlockEntity::Mod(saved)) =
            crate::save::decode_block_entity(&Tag::Compound(wire)).unwrap()
        else {
            panic!()
        };
        assert!(saved.nbt.contains_key(&FIELD.into()));
        assert_eq!(decoded.data, be.data);
        assert_eq!(
            decoded.nbt.get(&"cable".into()),
            be.nbt.get(&"cable".into())
        );
        assert!(!decoded.nbt.contains_key(&FIELD.into()));
        assert_eq!(be.nbt, original);
    }
    #[test]
    fn ae2_cable_wire_refuses_malformed_visuals() {
        let mut be = ModBlockEntity {
            id: "ae2:cable_bus".into(),
            nbt: Default::default(),
            items: vec![],
            data: vec![],
        };
        for data in [
            vec![],
            vec![0; 8],
            vec![0x414532, 64, 0, 0, 0, 0, 0, 0],
            vec![0x414532, 0, -1, 0, 0, 0, 0, 0],
            vec![0x414532, 0, 33, 0, 0, 0, 0, 0],
        ] {
            restore(&mut be, Some(&Tag::IntArray(data)));
            assert!(be.data.is_empty());
        }
    }
}
