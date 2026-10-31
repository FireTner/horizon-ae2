// SPDX-License-Identifier: LGPL-3.0-or-later
// DriveMenu RestrictedInputSlot / StorageCells built-in handled item families.
// Addon registry callbacks and the remaining portable/tool cell backends are
// separate work; GUI validity must not silently reject valid upstream cells.

pub fn handled_cell(id: &str) -> bool {
    let Some(id) = id.strip_prefix("ae2:") else {
        return false;
    };
    if matches!(
        id,
        "creative_storage_cell" | "matter_cannon" | "color_applicator"
    ) {
        return true;
    }
    [
        "item_storage_cell_",
        "fluid_storage_cell_",
        "portable_item_cell_",
        "portable_fluid_cell_",
    ]
    .into_iter()
    .any(|prefix| {
        id.strip_prefix(prefix)
            .is_some_and(|tier| matches!(tier, "1k" | "4k" | "16k" | "64k" | "256k"))
    })
}
pub fn drive_slot(id: &str, count: u32) -> (bool, u32) {
    (count > 0 && handled_cell(id), 1)
}

pub const WIDTH: f32 = 176.0;
pub const HEIGHT: f32 = 201.0;
pub const INVENTORY_Y: f32 = 117.0;
pub fn drive_position(slot: usize) -> Option<[f32; 2]> {
    (slot < 10).then_some([
        71.0 + (slot % 2) as f32 * 18.0,
        14.0 + (slot / 2) as f32 * 18.0,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_builtin_basic_portable_creative_and_tool_cells_are_valid() {
        for prefix in [
            "item_storage_cell_",
            "fluid_storage_cell_",
            "portable_item_cell_",
            "portable_fluid_cell_",
        ] {
            for tier in ["1k", "4k", "16k", "64k", "256k"] {
                assert!(handled_cell(&format!("ae2:{prefix}{tier}")));
            }
        }
        for id in ["creative_storage_cell", "matter_cannon", "color_applicator"] {
            assert!(handled_cell(&format!("ae2:{id}")));
        }
        for id in [
            "minecraft:stone",
            "ae2:view_cell",
            "ae2:spatial_storage_cell_2",
            "ae2:item_storage_cell_2k",
            "other:item_storage_cell_1k",
        ] {
            assert!(!handled_cell(id));
        }
        assert_eq!(drive_slot("ae2:item_storage_cell_1k", 0), (false, 1));
        assert_eq!(drive_slot("ae2:item_storage_cell_1k", 64), (true, 1));
    }
    #[test]
    fn original_drive_layout_has_two_columns_five_rows_and_no_extra_slots() {
        assert_eq!(drive_position(0), Some([71.0, 14.0]));
        assert_eq!(drive_position(9), Some([89.0, 86.0]));
        assert_eq!(drive_position(10), None);
        assert_eq!(INVENTORY_Y, HEIGHT - 84.0);
    }
}
