// SPDX-License-Identifier: LGPL-3.0-or-later
// Native AEKey/GenericStack component codec and basic storage-cell adapter.
// Uses AE2's actual id/components/#t/# and ae2:storage_cell_* layout.
// Unsupported/corrupt entries fail atomically; their original component is kept.

use super::ae2_backend::{CellBackend, CreativeCell};
use super::ae2_keys::{FuzzyMode, GenericStack, Key, KeyProperties, KeyType};
use super::ae2_storage::{Cell, CellLayout, Tier};
use horizon_client::{
    item_data,
    items::{ItemStack, Items},
    text::{Style, Text},
};
use horizon_world::nbt::{self, Compound, JavaString, Limits, Tag};
use std::collections::{BTreeMap, BTreeSet};

pub const INVENTORY: &str = "ae2:storage_cell_inv";
const CONFIG: &str = "ae2:storage_cell_config_inv";

pub fn cell_type(id: &str) -> Option<(Tier, KeyType, CellLayout)> {
    // Tools do not expose a StorageTier. These internal markers retain the
    // shared codec identity check; their explicit layouts control every limit.
    match id {
        "ae2:matter_cannon" => return Some((Tier::K1, KeyType::Item, CellLayout::matter_cannon())),
        "ae2:color_applicator" => {
            return Some((Tier::K4, KeyType::Item, CellLayout::color_applicator()));
        }
        _ => {}
    }
    let rest = id.strip_prefix("ae2:")?;
    let (kind, tier, portable) = if let Some(t) = rest.strip_prefix("item_storage_cell_") {
        (KeyType::Item, t, false)
    } else if let Some(t) = rest.strip_prefix("fluid_storage_cell_") {
        (KeyType::Fluid, t, false)
    } else if let Some(t) = rest.strip_prefix("portable_item_cell_") {
        (KeyType::Item, t, true)
    } else {
        (
            KeyType::Fluid,
            rest.strip_prefix("portable_fluid_cell_")?,
            true,
        )
    };
    let tier = match tier {
        "1k" => Tier::K1,
        "4k" => Tier::K4,
        "16k" => Tier::K16,
        "64k" => Tier::K64,
        "256k" => Tier::K256,
        _ => return None,
    };
    let layout = if portable {
        CellLayout::portable(tier, kind)
    } else {
        CellLayout::basic(tier, kind)
    };
    Some((tier, kind, layout))
}

fn int(tag: Option<&Tag>) -> Option<i64> {
    match tag? {
        Tag::Byte(v) => Some(i64::from(*v)),
        Tag::Short(v) => Some(i64::from(*v)),
        Tag::Int(v) => Some(i64::from(*v)),
        Tag::Long(v) => Some(*v),
        _ => None,
    }
}
fn text(tag: Option<&Tag>) -> Option<String> {
    match tag? {
        Tag::String(s) => s.to_string().ok(),
        _ => None,
    }
}
fn component<'a>(stack: &'a ItemStack, name: &str) -> Option<&'a Tag> {
    stack.data()?.other(name)
}
fn encoded_components(tag: &Tag) -> Result<Vec<u8>, String> {
    match tag {
        Tag::Compound(map) if map.is_empty() => Ok(vec![]),
        Tag::Compound(_) => nbt::write_network(tag, Limits::saved()).map_err(|e| e.to_string()),
        _ => Err("key components must be a compound".into()),
    }
}

pub fn key_components(key: &Key) -> Result<Tag, String> {
    if key.components().is_empty() {
        return Ok(Tag::Compound(Compound::new()));
    }
    let (tag, used) =
        nbt::read_network(key.components(), Limits::saved()).map_err(|e| e.to_string())?;
    if used != key.components().len() || !matches!(tag, Tag::Compound(_)) {
        return Err("invalid key component encoding".into());
    }
    Ok(tag)
}

fn key_with_properties(stack: &ItemStack, mut properties: KeyProperties) -> Result<Key, String> {
    if stack.count == 0 {
        return Err("empty item stack".into());
    }
    properties.damage = stack.damage as i32;
    let components =
        item_data::encode_components(stack).unwrap_or_else(|| Tag::Compound(Compound::new()));
    if let Some(value) = int(components.get("minecraft:max_damage")) {
        properties.max_damage = value.clamp(0, i64::from(i32::MAX)) as i32;
    }
    if let Some(value) = int(components.get("minecraft:max_stack_size")) {
        properties.max_stack = value.clamp(1, 99) as u32;
    }
    if tool_cell(&stack.item) {
        // Both tools explicitly override storableInStorageCell to true.
        properties.can_fit_inside_cell = true;
    } else if cell_type(&stack.item).is_some() {
        properties.can_fit_inside_cell = component(stack, INVENTORY)
            .is_none_or(|tag| matches!(tag,Tag::List(values) if values.is_empty()));
    } else if &*stack.item == "ae2:creative_storage_cell" {
        properties.can_fit_inside_cell = component(stack, CONFIG)
            .is_none_or(|tag| matches!(tag,Tag::List(values) if values.iter().take(63).all(|entry| matches!(entry,Tag::Compound(map) if map.is_empty()))));
    }
    Key::new(
        KeyType::Item,
        stack.item.to_string(),
        encoded_components(&components)?,
        properties,
    )
    .ok_or_else(|| "invalid item key".into())
}

pub fn item_key(stack: &ItemStack, items: &Items) -> Result<Key, String> {
    let info = items.get(&stack.item).ok_or("unknown item key")?;
    key_with_properties(
        stack,
        KeyProperties {
            max_damage: info.max_damage as i32,
            max_stack: info.max_stack,
            ..Default::default()
        },
    )
}

pub fn key_to_item(key: &Key, amount: u32) -> Result<ItemStack, String> {
    if key.kind() != KeyType::Item || amount == 0 {
        return Err("not a positive item stack".into());
    }
    let components = key_components(key)?;
    let (damage, extra) = item_data::decode_components(Some(&components));
    Ok(ItemStack {
        item: key.id().into(),
        count: amount,
        damage,
        extra,
    })
}

pub fn generic_from_tag(
    tag: &Tag,
    resolve: &impl Fn(KeyType, &str) -> Option<KeyProperties>,
) -> Result<GenericStack, String> {
    let kind = text(tag.get("#t"))
        .and_then(|s| KeyType::from_codec_id(&s))
        .ok_or("unknown AE key type")?;
    let id = text(tag.get("id")).ok_or("missing key id")?;
    let properties = resolve(kind, &id).ok_or("unresolved key registry entry")?;
    let empty = Tag::Compound(Compound::new());
    let components = tag.get("components").unwrap_or(&empty);
    if !matches!(components, Tag::Compound(_)) {
        return Err("components must be a compound".into());
    }
    let what = if kind == KeyType::Item {
        let (damage, extra) = item_data::decode_components(Some(components));
        key_with_properties(
            &ItemStack {
                item: id.into(),
                count: 1,
                damage,
                extra,
            },
            properties,
        )?
    } else {
        Key::new(kind, id, encoded_components(components)?, properties)
            .ok_or("invalid fluid key")?
    };
    Ok(GenericStack {
        what,
        amount: int(tag.get("#")).ok_or("missing stack amount")?,
    })
}

pub fn generic_to_tag(stack: &GenericStack) -> Result<Tag, String> {
    let mut map = Compound::new();
    map.insert(
        JavaString::from("#t"),
        Tag::String(stack.what.kind().codec_id().into()),
    );
    map.insert(JavaString::from("id"), Tag::String(stack.what.id().into()));
    map.insert(JavaString::from("#"), Tag::Long(stack.amount));
    let components = key_components(&stack.what)?;
    if !matches!(&components,Tag::Compound(map) if map.is_empty()) {
        map.insert(JavaString::from("components"), components);
    }
    Ok(Tag::Compound(map))
}

fn upgrades(stack: &ItemStack) -> BTreeSet<String> {
    match component(stack, "ae2:upgrades") {
        Some(Tag::List(list)) => list
            .iter()
            .filter(|entry| int(entry.get("count")).unwrap_or(1) > 0)
            .filter_map(|entry| text(entry.get("id")))
            .collect(),
        _ => BTreeSet::new(),
    }
}

fn config_keys(
    stack: &ItemStack,
    resolve: &impl Fn(KeyType, &str) -> Option<KeyProperties>,
) -> Result<BTreeSet<Key>, String> {
    let mut keys = BTreeSet::new();
    match component(stack, CONFIG) {
        None => {}
        Some(Tag::List(list)) => {
            // CellConfig uses a 63-slot ConfigInventory. Empty compounds are
            // nullable GenericStack slots; trailing entries are not loaded.
            for entry in list.iter().take(63) {
                if matches!(entry,Tag::Compound(map) if map.is_empty()) {
                    continue;
                }
                keys.insert(generic_from_tag(entry, resolve)?.what);
            }
        }
        _ => return Err("invalid cell partition".into()),
    }
    Ok(keys)
}

pub fn tool_cell(id: &str) -> bool {
    matches!(id, "ae2:matter_cannon" | "ae2:color_applicator")
}
pub fn load_restricted_backend(
    stack: &ItemStack,
    resolve: &impl Fn(KeyType, &str) -> Option<KeyProperties>,
    allowed: BTreeSet<String>,
) -> Result<CellBackend<Key>, String> {
    if !tool_cell(&stack.item) {
        return Err("not a tool cell".into());
    }
    load_cell(stack, resolve).map(|cell| CellBackend::restricted(cell, allowed))
}
pub fn load_backend(
    stack: &ItemStack,
    resolve: &impl Fn(KeyType, &str) -> Option<KeyProperties>,
) -> Result<CellBackend<Key>, String> {
    if stack.count == 0 {
        Err("empty cell stack".into())
    } else if tool_cell(&stack.item) {
        Err("tool cell requires recipe/tag admission context".into())
    } else if &*stack.item == "ae2:creative_storage_cell" {
        Ok(CreativeCell::new(config_keys(stack, resolve)?).into())
    } else {
        load_cell(stack, resolve).map(Into::into)
    }
}
pub fn persist_backend(stack: &mut ItemStack, cell: &CellBackend<Key>) -> Result<(), String> {
    match cell {
        CellBackend::Basic(cell) | CellBackend::Restricted { cell, .. } => {
            persist_cell(stack, cell)
        }
        CellBackend::Creative(_) if &*stack.item == "ae2:creative_storage_cell" => Ok(()),
        _ => Err("creative cell identity changed during operation".into()),
    }
}

pub fn load_cell(
    stack: &ItemStack,
    resolve: &impl Fn(KeyType, &str) -> Option<KeyProperties>,
) -> Result<Cell<Key>, String> {
    if stack.count == 0 {
        return Err("empty cell stack".into());
    }
    let (tier, kind, layout) = cell_type(&stack.item).ok_or("not a basic cell")?;
    let mut contents = BTreeMap::new();
    match component(stack, INVENTORY) {
        None => {}
        Some(Tag::List(list)) => {
            for entry in list {
                let entry = generic_from_tag(entry, resolve)?;
                if entry.amount <= 0 || contents.insert(entry.what, entry.amount as u64).is_some() {
                    return Err("invalid/duplicate cell entry".into());
                }
            }
        }
        _ => return Err("invalid cell inventory component".into()),
    }
    let mut cell =
        Cell::restore_with_layout(tier, kind, layout, contents).map_err(str::to_owned)?;
    let partition = config_keys(stack, resolve)?
        .into_iter()
        .filter(|key| key.kind() == kind)
        .collect();
    let cards = upgrades(stack);
    cell.configure(
        partition,
        cards.contains("ae2:inverter_card"),
        cards.contains("ae2:equal_distribution_card"),
        cards.contains("ae2:void_card"),
    );
    if cards.contains("ae2:fuzzy_card") {
        let mode = text(component(stack, "ae2:storage_cell_fuzzy_mode"))
            .and_then(|name| FuzzyMode::from_name(&name))
            .unwrap_or_default();
        cell.set_fuzzy(Some(mode));
    }
    Ok(cell)
}

/// Only commit the contents component. Name, upgrades, partition, wire metadata
/// and all unrelated components stay on the same ItemStack.
pub fn persist_cell(stack: &mut ItemStack, cell: &Cell<Key>) -> Result<(), String> {
    let (tier, kind, layout) = cell_type(&stack.item).ok_or("not a basic cell")?;
    if cell.tier() != tier || cell.kind() != kind || cell.layout() != layout {
        return Err("cell identity changed during operation".into());
    }
    let values = cell
        .stacks()
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(what, n)| {
            generic_to_tag(&GenericStack {
                what: what.clone(),
                amount: *n as i64,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    stack
        .data_mut()
        .set_other(INVENTORY, (!values.is_empty()).then_some(Tag::List(values)));
    Ok(())
}

pub fn resolve_key(items: &Items, kind: KeyType, id: &str) -> Option<KeyProperties> {
    match kind {
        KeyType::Item => items.get(id).map(|i| KeyProperties {
            max_damage: i.max_damage as i32,
            max_stack: i.max_stack,
            ..Default::default()
        }),
        KeyType::Fluid => {
            matches!(id, "minecraft:water" | "minecraft:lava").then_some(KeyProperties::default())
        }
    }
}

pub(super) fn tooltip(game: &super::Game<'_>, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    if cell_type(&stack.item).is_none() {
        return;
    }
    let items = game.items();
    if item_key(stack, items).is_err() {
        return;
    }
    let resolve = |kind, id: &str| resolve_key(items, kind, id);
    if let Ok(cell) = load_cell(stack, &resolve) {
        let ratio = |used: u64, max: u64| {
            let p = 1.0 - used as f64 / max as f64;
            let red = (255.0 * (2.0 - 2.0 * p).clamp(0.0, 1.0)) as u32;
            let green = (255.0 * (2.0 * p).clamp(0.0, 1.0)) as u32;
            Text::literal(used.to_string())
                .with_style(&Style {
                    color: Some((red << 16) | (green << 8)),
                    italic: Some(false),
                    ..Default::default()
                })
                .append(" ")
                .append(Text::translate("gui.ae2.Of", &[]))
                .append(" ")
                .append(Text::literal(max.to_string()).with_style(&Style {
                    color: Some(0x886eff),
                    italic: Some(false),
                    ..Default::default()
                }))
        };
        lines.push((
            Text::translate(
                "gui.ae2.BytesUsed",
                &[ratio(cell.used_bytes(), cell.layout().bytes())],
            ),
            u32::MAX,
        ));
        lines.push((
            ratio(
                cell.stored_types() as u64,
                cell.layout().total_types() as u64,
            )
            .append(" ")
            .append(Text::translate("gui.ae2.Types", &[])),
            u32::MAX,
        ));
        if cell.is_preformatted() {
            let included = if cell.is_inverted() {
                "gui.ae2.Excluded"
            } else {
                "gui.ae2.Included"
            };
            let precision = if cell.fuzzy_mode().is_some() {
                "gui.ae2.Fuzzy"
            } else {
                "gui.ae2.Precise"
            };
            lines.push((
                Text::translate("gui.ae2.Partitioned", &[])
                    .append(" - ")
                    .append(Text::translate(included, &[]))
                    .append(" ")
                    .append(Text::translate(precision, &[])),
                u32::MAX,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(id: &str) -> ItemStack {
        ItemStack {
            item: id.into(),
            count: 1,
            damage: 0,
            extra: None,
        }
    }
    fn resolve(_: KeyType, id: &str) -> Option<KeyProperties> {
        (!id.starts_with("missing:")).then_some(KeyProperties::default())
    }
    #[test]
    fn tool_backend_needs_admission_and_preserves_private_energy_and_nested_contents() {
        let paint = Key::new(
            KeyType::Item,
            "ae2:red_paint_ball".to_owned(),
            vec![],
            Default::default(),
        )
        .unwrap();
        for id in ["ae2:matter_cannon", "ae2:color_applicator"] {
            let mut stack = item(id);
            stack
                .data_mut()
                .set_other("ae2:stored_energy", Some(Tag::Double(1234.0f64.to_bits())));
            let energy = component(&stack, "ae2:stored_energy").cloned();
            assert!(load_backend(&stack, &resolve).is_err());
            let mut cell = load_restricted_backend(
                &stack,
                &resolve,
                BTreeSet::from(["ae2:red_paint_ball".to_owned()]),
            )
            .unwrap();
            assert_eq!(cell.insert(paint.clone(), 17, true), 17);
            persist_backend(&mut stack, &cell).unwrap();
            assert_eq!(component(&stack, "ae2:stored_energy"), energy.as_ref());
            let mut loaded = load_restricted_backend(
                &stack,
                &resolve,
                BTreeSet::from(["ae2:red_paint_ball".to_owned()]),
            )
            .unwrap();
            assert_eq!(loaded.extract(&paint, 17, false), 17);
            let tool_key = key_with_properties(
                &stack,
                KeyProperties {
                    can_fit_inside_cell: false,
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(tool_key.properties().can_fit_inside_cell);
            let mut ordinary = load_cell(&item("ae2:item_storage_cell_1k"), &resolve).unwrap();
            assert_eq!(ordinary.insert(tool_key, 1, true), 1);
        }
    }
    #[test]
    fn zero_count_stacks_never_become_live_storage_handlers() {
        for id in [
            "ae2:item_storage_cell_1k",
            "ae2:portable_fluid_cell_4k",
            "ae2:creative_storage_cell",
            "ae2:matter_cannon",
            "ae2:color_applicator",
        ] {
            let mut stack = item(id);
            stack.count = 0;
            assert!(load_backend(&stack, &resolve).is_err());
            assert!(load_cell(&stack, &resolve).is_err());
        }
    }
    #[test]
    fn creative_backend_uses_configured_item_and_fluid_keys_and_persists_nothing() {
        let mut stack = item("ae2:creative_storage_cell");
        let iron = Key::new(
            KeyType::Item,
            "minecraft:iron_ingot".to_owned(),
            vec![],
            Default::default(),
        )
        .unwrap();
        let water = Key::new(
            KeyType::Fluid,
            "minecraft:water".to_owned(),
            vec![],
            Default::default(),
        )
        .unwrap();
        stack.data_mut().set_other(
            CONFIG,
            Some(Tag::List(vec![
                generic_to_tag(&GenericStack {
                    what: iron.clone(),
                    amount: 0,
                })
                .unwrap(),
                Tag::Compound(Compound::new()),
                generic_to_tag(&GenericStack {
                    what: water.clone(),
                    amount: 0,
                })
                .unwrap(),
                generic_to_tag(&GenericStack {
                    what: iron.clone(),
                    amount: 0,
                })
                .unwrap(),
            ])),
        );
        stack
            .data_mut()
            .set_other("test:opaque", Some(Tag::Byte(9)));
        let original = stack.clone();
        let mut cell = load_backend(&stack, &resolve).unwrap();
        assert_eq!(cell.stacks().len(), 2);
        assert_eq!(cell.stacks()[&iron], i32::MAX as u64);
        for key in [iron, water] {
            assert_eq!(
                cell.extract(&key, 1_000_000_000_000, false),
                1_000_000_000_000
            );
            assert_eq!(
                cell.extract(&key, 1_000_000_000_000, true),
                1_000_000_000_000
            );
            assert_eq!(cell.insert(key, 1_000_000_000_000, true), 1_000_000_000_000);
        }
        persist_backend(&mut stack, &cell).unwrap();
        assert_eq!(stack, original);
        assert!(
            !key_with_properties(&stack, Default::default())
                .unwrap()
                .properties()
                .can_fit_inside_cell
        );
        stack.item = "ae2:item_storage_cell_1k".into();
        assert!(persist_backend(&mut stack, &cell).is_err());
    }
    #[test]
    fn config_inventory_ignores_trailing_slots_and_empty_placeholders_can_be_nested() {
        let mut stack = item("ae2:creative_storage_cell");
        let mut entries = vec![Tag::Compound(Compound::new()); 63];
        entries.push(Tag::compound([
            ("#t", Tag::String("ae2:i".into())),
            ("id", Tag::String("missing:ignored_after_slot_63".into())),
            ("#", Tag::Long(0)),
        ]));
        stack.data_mut().set_other(CONFIG, Some(Tag::List(entries)));
        let cell = load_backend(&stack, &resolve).unwrap();
        assert!(cell.stacks().is_empty());
        assert_eq!(
            cell.state(),
            super::super::ae2_storage::CellState::TypesFull
        );
        assert!(
            key_with_properties(&stack, Default::default())
                .unwrap()
                .properties()
                .can_fit_inside_cell
        );
        stack.item = "ae2:item_storage_cell_1k".into();
        assert!(!load_cell(&stack, &resolve).unwrap().is_preformatted());
    }
    #[test]
    fn portable_cell_codec_uses_its_layout_and_preserves_energy_components() {
        for (prefix, kind) in [
            ("portable_item_cell_", KeyType::Item),
            ("portable_fluid_cell_", KeyType::Fluid),
        ] {
            for (name, tier) in ["1k", "4k", "16k", "64k", "256k"]
                .into_iter()
                .zip(Tier::ALL)
            {
                let mut stack = item(&format!("ae2:{prefix}{name}"));
                stack
                    .data_mut()
                    .set_other("ae2:stored_energy", Some(Tag::Double(123.0f64.to_bits())));
                stack.data_mut().set_other(
                    "ae2:energy_capacity",
                    Some(Tag::Double(20000.0f64.to_bits())),
                );
                let original = stack.clone();
                let mut cell = load_cell(&stack, &resolve).unwrap();
                assert_eq!(cell.layout(), CellLayout::portable(tier, kind));
                let key = Key::new(
                    kind,
                    if kind == KeyType::Item {
                        "minecraft:iron_ingot"
                    } else {
                        "minecraft:water"
                    }
                    .to_owned(),
                    vec![],
                    Default::default(),
                )
                .unwrap();
                assert_eq!(cell.insert(key.clone(), 1000, true), 1000);
                persist_cell(&mut stack, &cell).unwrap();
                let mut loaded = load_cell(&stack, &resolve).unwrap();
                assert_eq!(loaded.extract(&key, 1000, true), 1000);
                persist_cell(&mut stack, &loaded).unwrap();
                assert!(stack.same_item_same_components(&original));
            }
        }
    }
    #[test]
    fn swapping_basic_and_portable_identity_cannot_persist_under_wrong_capacity() {
        let mut stack = item("ae2:portable_item_cell_1k");
        let portable = load_cell(&stack, &resolve).unwrap();
        stack.item = "ae2:item_storage_cell_1k".into();
        assert!(persist_cell(&mut stack, &portable).is_err());
    }
    #[test]
    fn typed_unknown_components_and_nested_data_survive_roundtrip() {
        let mut input = item("minecraft:diamond_pickaxe");
        input.damage = 77;
        input.data_mut().set_other(
            "test:payload",
            Some(Tag::compound([
                ("small", Tag::Byte(1)),
                ("large", Tag::Long(9_223_372_036_854_000)),
                ("bytes", Tag::ByteArray(vec![0, 255, 128])),
                ("nested", Tag::List(vec![Tag::String("тест".into())])),
            ])),
        );
        let key = key_with_properties(&input, Default::default()).unwrap();
        let generic = GenericStack {
            what: key,
            amount: 1_000_000_000_000,
        };
        let tag = generic_to_tag(&generic).unwrap();
        let back = generic_from_tag(&tag, &resolve).unwrap();
        assert_eq!(back, generic);
        let stack = key_to_item(&back.what, 42).unwrap();
        assert!(stack.same_item_same_components(&input));
        assert_eq!(stack.count, 42);
    }
    #[test]
    fn cell_persistence_keeps_name_upgrades_and_other_components() {
        let mut stack = item("ae2:item_storage_cell_1k");
        stack.data_mut().set_other(
            "minecraft:custom_name",
            Some(Tag::String("Named Cell".into())),
        );
        stack.data_mut().set_other(
            "ae2:upgrades",
            Some(Tag::List(vec![Tag::compound([
                ("id", Tag::String("ae2:void_card".into())),
                ("count", Tag::Int(1)),
            ])])),
        );
        stack
            .data_mut()
            .set_other("test:custom", Some(Tag::Int(123)));
        let original = stack.clone();
        let key = key_with_properties(&item("minecraft:stone"), Default::default()).unwrap();
        let mut cell = load_cell(&stack, &resolve).unwrap();
        cell.insert(key.clone(), 8000, true);
        persist_cell(&mut stack, &cell).unwrap();
        let loaded = load_cell(&stack, &resolve).unwrap();
        assert_eq!(loaded.stored_count(), 8000);
        assert_eq!(component(&stack, "test:custom"), Some(&Tag::Int(123)));
        let mut loaded = loaded;
        loaded.extract(&key, 8000, true);
        persist_cell(&mut stack, &loaded).unwrap();
        assert!(component(&stack, INVENTORY).is_none());
        assert!(stack.same_item_same_components(&original));
        assert_eq!(component(&stack, "test:custom"), Some(&Tag::Int(123)));
    }
    #[test]
    fn malformed_or_unknown_keys_never_replace_the_original_cell() {
        for tag in [
            Tag::Int(42),
            Tag::List(vec![Tag::compound([
                ("#t", Tag::String("ae2:i".into())),
                ("id", Tag::String("missing:item".into())),
                ("#", Tag::Long(5)),
            ])]),
        ] {
            let mut stack = item("ae2:item_storage_cell_1k");
            stack.data_mut().set_other(INVENTORY, Some(tag));
            let original = stack.clone();
            assert!(load_cell(&stack, &resolve).is_err());
            assert_eq!(stack, original);
        }
    }
    #[test]
    fn nonempty_nested_basic_cells_are_rejected_and_empty_ones_fit() {
        let mut stack = item("ae2:item_storage_cell_1k");
        assert!(
            key_with_properties(&stack, Default::default())
                .unwrap()
                .properties()
                .can_fit_inside_cell
        );
        stack.data_mut().set_other(
            INVENTORY,
            Some(Tag::List(vec![Tag::compound([(
                "id",
                Tag::String("minecraft:stone".into()),
            )])])),
        );
        assert!(
            !key_with_properties(&stack, Default::default())
                .unwrap()
                .properties()
                .can_fit_inside_cell
        );
    }
    #[test]
    fn typed_nbt_distinguishes_byte_and_int_and_preserves_negative_generic_counts() {
        let mut a = item("minecraft:stone");
        let mut b = a.clone();
        a.data_mut().set_other("test:value", Some(Tag::Byte(1)));
        b.data_mut().set_other("test:value", Some(Tag::Int(1)));
        let a = key_with_properties(&a, Default::default()).unwrap();
        let b = key_with_properties(&b, Default::default()).unwrap();
        assert_ne!(a, b);
        let stack = GenericStack {
            what: a,
            amount: -42,
        };
        assert_eq!(
            generic_from_tag(&generic_to_tag(&stack).unwrap(), &resolve).unwrap(),
            stack
        );
    }

    #[test]
    fn fuzzy_distribution_cards_are_read_from_original_component_layout() {
        let mut stack = item("ae2:item_storage_cell_1k");
        let filter = key_with_properties(
            &item("minecraft:diamond_pickaxe"),
            KeyProperties {
                max_damage: 100,
                ..Default::default()
            },
        )
        .unwrap();
        stack.data_mut().set_other(
            CONFIG,
            Some(Tag::List(vec![
                generic_to_tag(&GenericStack {
                    what: filter,
                    amount: 1,
                })
                .unwrap(),
            ])),
        );
        stack.data_mut().set_other(
            "ae2:upgrades",
            Some(Tag::List(
                ["ae2:fuzzy_card", "ae2:equal_distribution_card"]
                    .into_iter()
                    .map(|id| {
                        Tag::compound([("id", Tag::String(id.into())), ("count", Tag::Int(1))])
                    })
                    .collect(),
            )),
        );
        stack.data_mut().set_other(
            "ae2:storage_cell_fuzzy_mode",
            Some(Tag::String("PERCENT_75".into())),
        );
        let resolve = |_: KeyType, _: &str| {
            Some(KeyProperties {
                max_damage: 100,
                ..Default::default()
            })
        };
        let mut loaded = load_cell(&stack, &resolve).unwrap();
        assert_eq!(loaded.max_per_type(), 67);
        let mut candidate = item("minecraft:diamond_pickaxe");
        candidate.damage = 26;
        let key = key_with_properties(&candidate, resolve(KeyType::Item, "").unwrap()).unwrap();
        assert_eq!(loaded.insert(key, 1, true), 0);
        candidate.damage = 25;
        let key = key_with_properties(&candidate, resolve(KeyType::Item, "").unwrap()).unwrap();
        assert_eq!(loaded.insert(key, 100, true), 67);
    }

    #[test]
    fn fluid_key_roundtrip_does_not_become_an_item_and_preserves_typed_components() {
        let source = Tag::compound([
            ("#t", Tag::String("ae2:f".into())),
            ("id", Tag::String("minecraft:water".into())),
            ("#", Tag::Long(8_128_000)),
            (
                "components",
                Tag::compound([("test:raw", Tag::String(JavaString(vec![0xd800])))]),
            ),
        ]);
        let stack = generic_from_tag(&source, &resolve).unwrap();
        assert_eq!(generic_to_tag(&stack).unwrap(), source);
        assert!(key_to_item(&stack.what, 1).is_err());
        let mut cell = item("ae2:fluid_storage_cell_1k");
        cell.data_mut()
            .set_other(INVENTORY, Some(Tag::List(vec![source])));
        let mut loaded = load_cell(&cell, &resolve).unwrap();
        assert_eq!(loaded.used_bytes(), 1024);
        assert_eq!(loaded.extract(&stack.what, 1000, true), 1000);
        persist_cell(&mut cell, &loaded).unwrap();
        assert_eq!(
            load_cell(&cell, &resolve).unwrap().stored_count(),
            8_127_000
        );
    }
}
