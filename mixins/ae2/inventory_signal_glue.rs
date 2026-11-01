// SPDX-License-Identifier: LGPL-3.0-or-later
use horizon_client::items::ItemStack;
use horizon_world::nbt::{Compound, JavaString};

pub(super) fn signal(id: &str, nbt: &Compound, native: &[Option<ItemStack>]) -> Option<i32> {
    let slots = match id {
        "ae2:charger" => 1,
        "ae2:drive" => 10,
        _ => return None,
    };
    let saved = match nbt.get(&JavaString::from("inv")) {
        Some(tag) => match super::ae2_inventory_codec::read_slots(tag, slots) {
            Ok(items) => Some(items),
            Err(_) => return Some(0), // Same explicit malformed gate as these native hosts.
        },
        None => None,
    };
    Some(super::ae2_inventory_signal::signal(
        saved.as_deref().unwrap_or(native).iter().map(|item| {
            item.as_ref()
                .filter(|item| &*item.item != "minecraft:air")
                .map_or(0, |item| item.count)
        }),
        slots,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_world::nbt::Tag;

    #[test]
    fn original_source_count_takes_precedence_and_is_not_mutated() {
        let original = [Some(ItemStack::new("minecraft:stone", 3))];
        let mut nbt = Compound::new();
        nbt.insert(
            JavaString::from("inv"),
            super::super::ae2_inventory_codec::write_slots(
                original.iter().map(Option::as_ref),
                None,
            ),
        );
        let before = nbt.clone();
        assert_eq!(signal("ae2:charger", &nbt, &[None]), Some(43));
        assert_eq!(nbt, before);
    }

    #[test]
    fn legacy_native_fallback_ignores_drive_transport_padding() {
        let mut native = vec![None; 10];
        native.extend(vec![Some(ItemStack::new("minecraft:stone", 99)); 8]);
        assert_eq!(signal("ae2:drive", &Compound::new(), &native), Some(0));
        native[0] = Some(ItemStack::new("minecraft:stone", 1));
        native[9] = Some(ItemStack::new("minecraft:stone", 1));
        assert_eq!(signal("ae2:drive", &Compound::new(), &native), Some(3));
    }

    #[test]
    fn malformed_original_does_not_reappear_as_a_full_native_echo() {
        let mut stack = Compound::new();
        stack.insert(
            JavaString::from("id"),
            Tag::String("minecraft:stone".into()),
        );
        stack.insert(JavaString::from("count"), Tag::Int(0));
        let mut inv = Compound::new();
        inv.insert(JavaString::from("item0"), Tag::Compound(stack));
        let mut nbt = Compound::new();
        nbt.insert(JavaString::from("inv"), Tag::Compound(inv));
        let before = nbt.clone();
        let echo = [Some(ItemStack::new("ae2:portable_item_cell_1k", 1))];
        assert_eq!(signal("ae2:charger", &nbt, &echo), Some(0));
        assert_eq!(nbt, before);
    }

    #[test]
    fn other_containers_keep_their_native_signal_rules() {
        assert_eq!(signal("minecraft:barrel", &Compound::new(), &[]), None);
        assert_eq!(signal("example:machine", &Compound::new(), &[]), None);
    }

    #[test]
    fn air_stacks_are_empty_even_with_a_positive_legacy_native_count() {
        let items = [Some(ItemStack::new("minecraft:air", 3))];
        assert_eq!(signal("ae2:charger", &Compound::new(), &items), Some(0));
        let mut nbt = Compound::new();
        nbt.insert(
            JavaString::from("inv"),
            super::super::ae2_inventory_codec::write_slots(items.iter().map(Option::as_ref), None),
        );
        assert_eq!(signal("ae2:charger", &nbt, &items), Some(0));
    }
}
