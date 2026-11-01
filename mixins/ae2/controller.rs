// SPDX-License-Identifier: LGPL-3.0-or-later
// ControllerBlock / ControllerBlockEntity registration and visual state rules.
pub const BLOCK: &str = "ae2:controller";
pub const CAPACITY: f64 = 8000.0;
pub const IDLE: f64 = 3.0;
pub const STORAGE_PRIORITY: i32 = 0; // IAEPowerStorage default, not capacity / 1000.
pub const STATES: [&str; 3] = ["offline", "online", "conflicted"];
pub const TYPES: [&str; 6] = [
    "block", "column_x", "column_y", "column_z", "inside_a", "inside_b",
];
pub const VARIANTS: usize = STATES.len() * TYPES.len();

pub fn status(powered: bool, conflict: bool) -> &'static str {
    if !powered {
        "offline"
    } else if conflict {
        "conflicted"
    } else {
        "online"
    }
}

// Neighbours use AE2/Horizon's Direction order: down/up/north/south/west/east.
pub fn render_type(pos: [i32; 3], neighbours: [bool; 6]) -> &'static str {
    let xx = neighbours[4] && neighbours[5];
    let yy = neighbours[0] && neighbours[1];
    let zz = neighbours[2] && neighbours[3];
    match (xx, yy, zz) {
        (true, false, false) => "column_x",
        (false, true, false) => "column_y",
        (false, false, true) => "column_z",
        (false, false, false) => "block",
        _ => {
            let parity = pos
                .into_iter()
                .fold(0_i32, |sum, c| sum.wrapping_add(c.wrapping_abs()))
                % 2;
            if parity == 0 { "inside_a" } else { "inside_b" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn power_is_required_before_conflict_becomes_visible() {
        assert_eq!(status(false, false), "offline");
        assert_eq!(status(false, true), "offline");
        assert_eq!(status(true, false), "online");
        assert_eq!(status(true, true), "conflicted");
    }
    #[test]
    fn connected_columns_and_interior_parity_match_source_axes() {
        assert_eq!(render_type([0; 3], [false; 6]), "block");
        assert_eq!(
            render_type([0; 3], [false, false, false, false, true, true]),
            "column_x"
        );
        assert_eq!(
            render_type([0; 3], [true, true, false, false, false, false]),
            "column_y"
        );
        assert_eq!(
            render_type([0; 3], [false, false, true, true, false, false]),
            "column_z"
        );
        assert_eq!(render_type([-1, 2, -3], [true; 6]), "inside_a");
        assert_eq!(render_type([-1, 2, -4], [true; 6]), "inside_b");
        assert_eq!(render_type([i32::MIN, i32::MAX, 1], [true; 6]), "inside_a");
    }
}
