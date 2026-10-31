// SPDX-License-Identifier: LGPL-3.0-or-later
// AppEngCellInventory/AppEngInternalInventory: Drive has one cell per slot.
// World-side helpers call this only for the original ten physical slots.
use horizon_client::{items::ItemStack, lan::menu::ae2_slot_rules};

pub(super) fn insert(
    target: &mut Option<ItemStack>,
    mut stack: ItemStack,
) -> (Option<ItemStack>, bool) {
    if target.is_some() || !ae2_slot_rules::drive_slot(&stack.item, stack.count).0 {
        return (Some(stack), false);
    }
    let mut one = stack.clone();
    one.count = 1;
    *target = Some(one);
    stack.count -= 1;
    ((stack.count > 0).then_some(stack), true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_world::nbt::{Compound, JavaString, Tag};
    #[test]
    fn insert_preserves_component_identity_and_counts_for_stacked_inputs() {
        let mut components = Compound::new();
        components.insert(JavaString::from("example:data"), Tag::Long(9));
        let mut stack = ItemStack::new("ae2:item_storage_cell_1k", 3);
        let (damage, extra) =
            horizon_client::item_data::decode_components(Some(&Tag::Compound(components)));
        stack.damage = damage;
        stack.extra = extra;
        let mut target = None;
        let (rest, changed) = insert(&mut target, stack.clone());
        assert!(changed);
        let target = target.unwrap();
        assert_eq!(target.count, 1);
        assert_eq!(rest.as_ref().unwrap().count, 2);
        assert!(target.same_item_same_components(&stack));
        assert!(rest.as_ref().unwrap().same_item_same_components(&stack));
        let mut occupied = Some(target.clone());
        let (rest, changed) = insert(&mut occupied, rest.unwrap());
        assert!(!changed);
        assert_eq!(occupied, Some(target));
        assert_eq!(rest.unwrap().count, 2);
    }
    #[test]
    fn reject_non_cell_and_zero_count_without_mutation_and_consume_single_cell() {
        let mut target = None;
        for stack in [
            ItemStack::new("minecraft:stone", 64),
            ItemStack::new("ae2:item_storage_cell_1k", 0),
        ] {
            let (rest, changed) = insert(&mut target, stack.clone());
            assert!(!changed);
            assert_eq!(rest, Some(stack));
            assert!(target.is_none());
        }
        let (rest, changed) = insert(&mut target, ItemStack::new("ae2:fluid_storage_cell_4k", 1));
        assert!(changed && rest.is_none());
        assert_eq!(target.unwrap().count, 1);
    }
}
