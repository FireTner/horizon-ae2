// SPDX-License-Identifier: LGPL-3.0-or-later
// AEBaseInvBlockEntity / modern ItemStack optional saved form. This adapter
// preserves malformed original data through a strict refusal, pending recovery.
use horizon_client::{
    items::ItemStack,
    save::{item_from_nbt, item_to_nbt},
};
use horizon_world::nbt::{Compound, JavaString, Tag};

pub(super) fn read_stack(tag: &Tag) -> Result<Option<ItemStack>, &'static str> {
    let Tag::Compound(item) = tag else {
        return Err("inventory stack must be a compound");
    };
    if item.is_empty() {
        return Ok(None);
    }
    let count = match item.get(&JavaString::from("count")) {
        None => 1,
        Some(Tag::Byte(v)) => i64::from(*v),
        Some(Tag::Short(v)) => i64::from(*v),
        Some(Tag::Int(v)) => i64::from(*v),
        Some(Tag::Long(v)) => *v,
        _ => return Err("invalid saved item count"),
    };
    if !(1..=99).contains(&count) {
        return Err("invalid saved item count");
    }
    let mut stack = item_from_nbt(tag).ok_or("invalid saved item stack")?;
    stack.count = count as u32; // Source defaults to one; legacy native Count is not this field.
    Ok(Some(stack))
}
pub(super) fn read_slots(tag: &Tag, slots: usize) -> Result<Vec<Option<ItemStack>>, &'static str> {
    let Tag::Compound(inv) = tag else {
        return Err("inventory must be a compound");
    };
    (0..slots)
        .map(|slot| {
            inv.get(&JavaString::from(format!("item{slot}").as_str()))
                .map_or(Ok(None), read_stack)
        })
        .collect()
}
pub(super) fn write_slots<'a>(
    items: impl IntoIterator<Item = Option<&'a ItemStack>>,
    previous: Option<&Tag>,
) -> Tag {
    let mut inv = match previous {
        Some(Tag::Compound(inv)) => inv.clone(),
        _ => Compound::new(),
    };
    for (slot, item) in items.into_iter().enumerate() {
        let tag = item.filter(|item| item.count > 0).map_or_else(
            || Tag::Compound(Compound::new()),
            |item| {
                let mut tag = item_to_nbt(item);
                if item.count == 1
                    && let Tag::Compound(map) = &mut tag
                {
                    map.remove(&JavaString::from("count"));
                }
                tag
            },
        );
        inv.insert(JavaString::from(format!("item{slot}").as_str()), tag);
    }
    Tag::Compound(inv)
}
pub(super) fn choose_slots(
    original: Option<&Tag>,
    previous: Option<&Tag>,
    native: &[Option<ItemStack>],
    observed: bool,
    blocked: bool,
    slots: usize,
) -> Result<Vec<Option<ItemStack>>, &'static str> {
    if let Some(original) = original
        && (!observed || Some(original) != previous || blocked)
    {
        read_slots(original, slots)
    } else {
        let mut out = native.iter().take(slots).cloned().collect::<Vec<_>>();
        out.resize(slots, None);
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nonpositive_source_count_never_becomes_horizons_clamped_item() {
        for count in [0, -1, 100] {
            let tag = Tag::compound([
                ("id", Tag::String("ae2:item_storage_cell_1k".into())),
                ("count", Tag::Int(count)),
            ]);
            if count <= 0 {
                assert_eq!(item_from_nbt(&tag).unwrap().count, 1);
            }
            assert!(read_stack(&tag).is_err());
        }
        let tag = Tag::compound([
            ("id", Tag::String("ae2:item_storage_cell_1k".into())),
            ("Count", Tag::Byte(7)),
        ]);
        assert_eq!(read_stack(&tag).unwrap().unwrap().count, 1);
    }
    #[test]
    fn blocked_original_is_not_overwritten_by_an_updated_native_echo() {
        let bad = Tag::compound([(
            "item0",
            Tag::compound([
                ("id", Tag::String("ae2:item_storage_cell_1k".into())),
                ("count", Tag::Int(0)),
            ]),
        )]);
        let native = vec![Some(ItemStack::new("ae2:item_storage_cell_4k", 1))];
        assert!(choose_slots(Some(&bad), Some(&bad), &native, true, true, 1).is_err());
        let repaired = write_slots(std::iter::once(native[0].as_ref()), None);
        assert_eq!(
            choose_slots(Some(&repaired), Some(&bad), &native, true, true, 1).unwrap(),
            native
        );
    }
    #[test]
    fn original_optional_count_empties_and_unknown_inventory_fields_roundtrip() {
        let items = [
            Some(ItemStack::new("ae2:charged_staff", 1)),
            None,
            Some(ItemStack::new("ae2:charged_staff", 2)),
        ];
        let previous = Tag::compound([("test:unknown", Tag::Long(19))]);
        let tag = write_slots(items.iter().map(Option::as_ref), Some(&previous));
        assert!(tag.get("item0").unwrap().get("count").is_none());
        assert_eq!(tag.get("item2").unwrap().get("count"), Some(&Tag::Int(2)));
        assert_eq!(tag.get("test:unknown"), Some(&Tag::Long(19)));
        assert_eq!(read_slots(&tag, 3).unwrap(), items);
    }
}
