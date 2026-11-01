// SPDX-License-Identifier: LGPL-3.0-or-later
// Horizon native transport for AE2 item components. The envelope exists only on
// the wire; decoded/saved stacks retain their original component names and types.
use crate::{item_data, items::ItemStack};
use horizon_protocol::{
    item::{Component, Stack, component},
    registry::Builtin,
};
use horizon_world::nbt::{Compound, Tag};
const ENVELOPE: &str = "horizon_ae2:components";

pub fn encode(item_id: &dyn Fn(&str) -> i32, source: &Option<ItemStack>) -> Option<Stack> {
    let source = source.as_ref()?;
    if !source.item.starts_with("ae2:") {
        return None;
    }
    let components =
        item_data::encode_components(source).unwrap_or_else(|| Tag::Compound(Compound::new()));
    let envelope = Tag::compound([(
        ENVELOPE,
        Tag::compound([
            ("version", Tag::Int(1)),
            ("item", Tag::String(source.item.as_ref().into())),
            ("components", components),
        ]),
    )]);
    let mut out = Stack::new(item_id(&source.item), source.count as i32);
    out.added
        .push((component::CUSTOM_DATA, Component::Nbt(envelope)));
    Some(out)
}
pub fn decode(stack: &Stack, builtin: &Builtin) -> Option<Tag> {
    let name = builtin.get("minecraft:item")?.name(stack.item)?;
    if !name.starts_with("ae2:") {
        return None;
    }
    let Component::Nbt(envelope) = stack.component(component::CUSTOM_DATA)? else {
        return None;
    };
    let data = envelope.get(ENVELOPE)?;
    if data.get("version") != Some(&Tag::Int(1))
        || data.get("item") != Some(&Tag::String(name.into()))
    {
        return None;
    }
    let components = data.get("components")?;
    matches!(components, Tag::Compound(_)).then(|| components.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_protocol::{
        buf::{Codec, Reader, Writer},
        item::{Hashed, Slot},
        registry::{Dynamic, Ids, SyncedNames},
    };
    fn registries() -> Builtin {
        let mut components = vec![String::new(); component::CONTAINER as usize + 1];
        for (index, name) in [
            (component::CUSTOM_DATA, "minecraft:custom_data"),
            (component::CONTAINER, "minecraft:container"),
            (component::BUNDLE_CONTENTS, "minecraft:bundle_contents"),
        ] {
            components[index as usize] = name.into();
        }
        Builtin {
            registries: [
                (
                    "minecraft:item".into(),
                    Ids::from_names(vec![
                        "ae2:portable_item_cell_1k".into(),
                        "minecraft:shulker_box".into(),
                        "minecraft:bundle".into(),
                    ]),
                ),
                (
                    "minecraft:data_component_type".into(),
                    Ids::from_names(components),
                ),
            ]
            .into(),
        }
    }
    fn source() -> ItemStack {
        let mut stack = ItemStack::new("ae2:portable_item_cell_1k", 1);
        stack
            .data_mut()
            .set_other("ae2:stored_energy", Some(Tag::Double(10000.0f64.to_bits())));
        stack.data_mut().set_other(
            "ae2:upgrades",
            Some(Tag::List(vec![Tag::compound([
                ("slot", Tag::Int(3)),
                (
                    "item",
                    Tag::compound([
                        ("id", Tag::String("ae2:energy_card".into())),
                        ("count", Tag::Int(1)),
                    ]),
                ),
            ])])),
        );
        stack.data_mut().set_other(
            "test:raw",
            Some(Tag::compound([
                ("byte", Tag::Byte(1)),
                ("long", Tag::Long(1)),
                ("bytes", Tag::ByteArray(vec![0, 255])),
            ])),
        );
        // A user's own custom data may contain our envelope name. It must stay
        // nested in the original component, never collide with the wire wrapper.
        stack.data_mut().set_other(
            "minecraft:custom_data",
            Some(Tag::compound([(ENVELOPE, Tag::Int(77))])),
        );
        stack
    }
    fn network(stack: &ItemStack, builtin: &Builtin) -> Stack {
        crate::lan::guest::stack_to_network(
            |id| builtin.get("minecraft:item").unwrap().id(id).unwrap(),
            |_, _| None,
            &Some(stack.clone()),
        )
        .unwrap()
    }
    fn roundtrip(stack: &ItemStack, builtin: &Builtin) -> (Stack, ItemStack) {
        let sent = network(stack, builtin);
        let mut writer = Writer::new();
        Slot::write(&Some(sent.clone()), &mut writer);
        let mut reader = Reader::new(&writer.data);
        let wire = Slot::read(&mut reader).unwrap().unwrap();
        assert_eq!(reader.remaining(), 0);
        let decoded = crate::item_sync::from_network(&wire, builtin, &Dynamic::default());
        (sent, decoded)
    }
    #[test]
    fn ae2_component_packet_roundtrip_keeps_energy_typed_unknowns_and_custom_data_without_leaking_envelope()
     {
        let builtin = registries();
        let source = source();
        let before = item_data::encode_components(&source);
        let (_, decoded) = roundtrip(&source, &builtin);
        assert_eq!(decoded.item, source.item);
        assert_eq!(decoded.count, source.count);
        assert_eq!(item_data::encode_components(&decoded), before);
        assert_eq!(item_data::encode_components(&source), before);
    }
    #[test]
    fn ae2_components_survive_nested_vanilla_shulker_and_bundle_packets() {
        let builtin = registries();
        let source = source();
        for id in ["minecraft:shulker_box", "minecraft:bundle"] {
            let mut outer = ItemStack::new(id, 1);
            if id.ends_with("shulker_box") {
                outer.data_mut().container = vec![(0, source.clone())];
            } else {
                outer.data_mut().bundle_contents = vec![source.clone()];
            }
            let (_, decoded) = roundtrip(&outer, &builtin);
            let nested = if id.ends_with("shulker_box") {
                &decoded.data().unwrap().container[0].1
            } else {
                &decoded.data().unwrap().bundle_contents[0]
            };
            assert_eq!(
                item_data::encode_components(nested),
                item_data::encode_components(&source)
            );
        }
    }
    #[test]
    fn ae2_packet_hash_roundtrip_is_stable_and_changes_when_energy_changes() {
        let builtin = registries();
        let dynamic = Dynamic::default();
        let names = SyncedNames {
            builtin: &builtin,
            dynamic: &dynamic,
        };
        let (sent, mut decoded) = roundtrip(&source(), &builtin);
        assert_eq!(
            Hashed::create(&sent, &names),
            Hashed::create(&network(&decoded, &builtin), &names)
        );
        decoded
            .data_mut()
            .set_other("ae2:stored_energy", Some(Tag::Double(10001.0f64.to_bits())));
        assert_ne!(
            Hashed::create(&sent, &names),
            Hashed::create(&network(&decoded, &builtin), &names)
        );
    }
    #[test]
    fn ae2_transport_ignores_vanilla_and_malformed_or_mismatched_wrappers() {
        let builtin = registries();
        let mut wire = network(&source(), &builtin);
        wire.item = 1;
        assert!(decode(&wire, &builtin).is_none());
        wire.item = 0;
        wire.added = vec![(
            component::CUSTOM_DATA,
            Component::Nbt(Tag::compound([(
                ENVELOPE,
                Tag::compound([
                    ("version", Tag::Int(2)),
                    ("item", Tag::String("ae2:portable_item_cell_1k".into())),
                    ("components", Tag::Compound(Compound::new())),
                ]),
            )])),
        )];
        assert!(decode(&wire, &builtin).is_none());
        assert!(encode(&|_| 0, &Some(ItemStack::new("minecraft:shulker_box", 1))).is_none());
    }
}
