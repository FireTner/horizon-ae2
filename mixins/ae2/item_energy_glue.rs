// SPDX-License-Identifier: LGPL-3.0-or-later
// Native IAEItemPowerStorage component bridge; addon item registry still pending.
use super::ae2_item_energy::{ItemEnergy, PoweredKind};
use super::ae2_upgrade_rules::Host;
use horizon_client::items::ItemStack;
use horizon_world::nbt::Tag;

pub(super) fn register_components(reg: &mut horizon_client::command::Registries) {
    for id in [
        "ae2:stored_energy",
        "ae2:energy_capacity",
        "ae2:upgrades",
        "ae2:storage_cell_inv",
        "ae2:storage_cell_config_inv",
        "ae2:storage_cell_fuzzy_mode",
    ] {
        if !reg.persistent_components.iter().any(|entry| entry == id) {
            reg.persistent_components.push(id.to_owned());
        }
        let registry = reg
            .by_registry
            .entry("minecraft:data_component_type".to_owned())
            .or_default();
        if !registry.ids.iter().any(|entry| entry == id) {
            registry.ids.push(id.to_owned());
        }
    }
}

pub(super) struct PoweredItem {
    pub energy: ItemEnergy,
    pub charge_rate: f64,
}
impl PoweredItem {
    pub fn is_chargeable(&self) -> bool {
        // Platform.isChargeable; all built-in AEBasePoweredItem hosts are WRITE.
        self.energy.maximum() > 0.0
    }
}
fn number(stack: &ItemStack, name: &str) -> Result<Option<f64>, String> {
    let Some(value) = stack.data().and_then(|data| data.other(name)) else {
        return Ok(None);
    };
    let value = match value {
        Tag::Byte(value) => *value as f64,
        Tag::Short(value) => *value as f64,
        Tag::Int(value) => *value as f64,
        Tag::Long(value) => *value as f64,
        Tag::Float(value) => f32::from_bits(*value) as f64,
        Tag::Double(value) => f64::from_bits(*value),
        _ => return Err(format!("invalid numeric component {name}")),
    };
    Ok(Some(value))
}
pub(super) fn load(stack: &ItemStack) -> Result<Option<PoweredItem>, String> {
    if stack.count == 0 {
        return Ok(None);
    }
    let Some(kind) = PoweredKind::from_item(&stack.item) else {
        return Ok(None);
    };
    let host = match kind {
        PoweredKind::Portable if stack.item.starts_with("ae2:portable_fluid_") => {
            Some(Host::PortableFluid)
        }
        PoweredKind::Portable => Some(Host::PortableItem),
        PoweredKind::ColorApplicator => Some(Host::ColorApplicator),
        PoweredKind::MatterCannon => Some(Host::MatterCannon),
        PoweredKind::WirelessTerminal => Some(Host::WirelessTerminal),
        _ => None,
    };
    let cards = host
        .map(|host| super::ae2_cells::upgrades_for_host(stack, host))
        .transpose()?
        .map_or(0, |cards| cards.energy_multiplier());
    Ok(Some(PoweredItem {
        energy: ItemEnergy::from_components(
            kind.default_capacity(),
            number(stack, "ae2:energy_capacity")?,
            number(stack, "ae2:stored_energy")?,
        ),
        charge_rate: kind.charge_rate(cards),
    }))
}
pub(super) fn chargeable(stack: &ItemStack) -> bool {
    load(stack)
        .ok()
        .flatten()
        .is_some_and(|item| item.is_chargeable())
}
pub(super) fn persist(stack: &mut ItemStack, item: &PoweredItem) {
    // Charging changes only stored_energy. Preserve the exact original capacity
    // override, upgrade items, storage/config contents and all other components.
    stack.data_mut().set_other(
        "ae2:stored_energy",
        item.energy
            .stored_component()
            .map(|value| Tag::Double(value.to_bits())),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_client::save::{item_from_nbt, item_to_nbt};
    #[test]
    fn command_registry_preserves_vanilla_and_energy_codec_requires_numbers() {
        let mut reg = horizon_client::command::Registries::default();
        reg.persistent_components
            .push("minecraft:damage".to_owned());
        register_components(&mut reg);
        let once = reg.persistent_components.clone();
        register_components(&mut reg);
        assert_eq!(reg.persistent_components, once);
        assert!(
            reg.persistent_components
                .iter()
                .any(|id| id == "minecraft:damage")
        );
        assert!(
            reg.ids("minecraft:data_component_type")
                .iter()
                .any(|id| id == "ae2:stored_energy")
        );
        for id in ["ae2:stored_energy", "ae2:energy_capacity"] {
            assert!(horizon_client::command::item::decode_component(id, &Tag::Int(20000)).is_ok());
            assert!(
                horizon_client::command::item::decode_component(
                    id,
                    &Tag::Double(f64::NAN.to_bits())
                )
                .is_ok()
            );
            assert!(
                horizon_client::command::item::decode_component(id, &Tag::String("invalid".into()))
                    .is_err()
            );
        }
    }
    #[test]
    fn all_sixteen_builtin_powered_items_have_original_defaults_and_charge_rates() {
        for (id, maximum, rate) in [
            ("ae2:charged_staff", 8000.0, 32.0),
            ("ae2:entropy_manipulator", 200000.0, 800.0),
            ("ae2:wireless_terminal", 1600000.0, 800.0),
            ("ae2:wireless_crafting_terminal", 1600000.0, 800.0),
            ("ae2:matter_cannon", 200000.0, 800.0),
            ("ae2:color_applicator", 20000.0, 80.0),
        ] {
            let stack = ItemStack::new(id, 1);
            let item = load(&stack).unwrap().unwrap();
            assert_eq!(item.energy.maximum(), maximum);
            assert_eq!(item.charge_rate, rate);
            assert!(chargeable(&stack));
        }
        for kind in ["item", "fluid"] {
            for tier in ["1k", "4k", "16k", "64k", "256k"] {
                let stack = ItemStack::new(&format!("ae2:portable_{kind}_cell_{tier}"), 1);
                let item = load(&stack).unwrap().unwrap();
                assert_eq!(item.energy.maximum(), 20000.0);
                assert_eq!(item.charge_rate, 80.0);
            }
        }
        assert!(!chargeable(&ItemStack::new("ae2:portable_item_cell_2k", 1)));
        assert!(!chargeable(&ItemStack::new("ae2:item_storage_cell_1k", 1)));
        assert!(!chargeable(&ItemStack::new("ae2:charged_staff", 0)));
    }
    #[test]
    fn charging_preserves_capacity_upgrade_nested_payload_and_roundtrips_original_components() {
        let mut stack = ItemStack::new("ae2:portable_item_cell_1k", 1);
        let upgrades = Tag::List(vec![Tag::compound([
            ("slot", Tag::Int(3)),
            ("item", item_to_nbt(&ItemStack::new("ae2:energy_card", 7))),
        ])]);
        stack
            .data_mut()
            .set_other("ae2:upgrades", Some(upgrades.clone()));
        let payload = Tag::List(vec![Tag::compound([
            ("id", Tag::String("minecraft:stone".into())),
            ("#", Tag::Long(123)),
        ])]);
        stack
            .data_mut()
            .set_other("ae2:storage_cell_inv", Some(payload.clone()));
        stack
            .data_mut()
            .set_other("ae2:energy_capacity", Some(Tag::Int(340000)));
        let mut powered = load(&stack).unwrap().unwrap();
        // Loading upgrades does not run the change callback or reset an override.
        assert_eq!(powered.energy.maximum(), 340000.0);
        assert_eq!(powered.charge_rate, 160.0);
        assert_eq!(powered.energy.inject(1600.0, true), 0.0);
        persist(&mut stack, &powered);
        assert_eq!(
            stack.data().unwrap().other("ae2:energy_capacity"),
            Some(&Tag::Int(340000))
        );
        assert_eq!(stack.data().unwrap().other("ae2:upgrades"), Some(&upgrades));
        assert_eq!(
            stack.data().unwrap().other("ae2:storage_cell_inv"),
            Some(&payload)
        );
        let restored = item_from_nbt(&item_to_nbt(&stack)).unwrap();
        assert_eq!(load(&restored).unwrap().unwrap().energy.current(), 1600.0);
        assert!(restored.same_item_same_components(&stack));
    }
    #[test]
    fn wireless_slot_bounds_and_invalid_energy_components_do_not_create_power() {
        let mut stack = ItemStack::new("ae2:wireless_crafting_terminal", 1);
        stack.data_mut().set_other(
            "ae2:upgrades",
            Some(Tag::List(
                (0..3)
                    .map(|slot| {
                        Tag::compound([
                            ("slot", Tag::Int(slot)),
                            ("item", item_to_nbt(&ItemStack::new("ae2:energy_card", 1))),
                        ])
                    })
                    .collect(),
            )),
        );
        assert_eq!(load(&stack).unwrap().unwrap().charge_rate, 2400.0);
        stack
            .data_mut()
            .set_other("ae2:energy_capacity", Some(Tag::Double(0.0f64.to_bits())));
        assert!(!chargeable(&stack));
        stack
            .data_mut()
            .set_other("ae2:energy_capacity", Some(Tag::String("invalid".into())));
        let original = stack.clone();
        assert!(load(&stack).is_err());
        assert!(!chargeable(&stack));
        assert_eq!(stack, original);
    }
}
