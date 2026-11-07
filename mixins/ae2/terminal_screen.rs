// SPDX-License-Identifier: LGPL-3.0-or-later
// AbstractTerminalPart.onUseWithoutItem opens the terminal menu on the server
// when the hand is empty. The control `screen` command reports `container`
// only after the client applies a supported OpenScreen. Assigning the server
// Game screen does not: the guest action restores it, and lan_open_container
// rejects ae2:cable_bus, so the window stays on `playing`.
//
// shortcut: minecraft:generic_9x1 is the OpenScreen the client draws as
// `container`. Replace it with MEStorageMenu when that menu can be shown.
// shortcut: nothing calls `open` until src/lib.rs and mixins/ae2/horizon-mixin.toml
// register this file and the right-click path sends `Menu` through native_open_menu.

/// Side-part id of the ME terminal.
pub const TERMINAL: &str = "ae2:terminal";
/// Menu `native_open_menu` can send. Rows select `minecraft:generic_9x{rows}`.
pub const MENU_ID: &str = "minecraft:generic_9x1";
pub const MENU_ROWS: u8 = 1;
/// `gui.ae2.Terminal` in the pinned language file.
pub const TITLE_KEY: &str = "gui.ae2.Terminal";

const FACES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];

/// One stored side part. `grid_node` is the part's `gn` tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Part<'a> {
    pub id: &'a str,
    pub grid_node: bool,
}

/// What the server sends so the client leaves `playing`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Menu {
    pub id: &'static str,
    pub rows: u8,
    pub title_key: &'static str,
}

/// Screen names the control `screen` command reports for this click.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Playing,
    Container,
}

impl Screen {
    pub fn label(self) -> &'static str {
        match self {
            Self::Playing => "playing",
            Self::Container => "container",
        }
    }
}

/// Empty-hand use of the single terminal stored on `face`.
/// Other faces, missing nodes, and a second part on the same face do not open.
pub fn open(hand_empty: bool, face: &str, parts: &[(&str, Part<'_>)]) -> Option<Menu> {
    if !hand_empty || !FACES.contains(&face) {
        return None;
    }
    let mut found = None;
    for (side, part) in parts {
        if *side != face {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(*part);
    }
    let part = found?;
    if part.id != TERMINAL || !part.grid_node {
        return None;
    }
    Some(Menu {
        id: MENU_ID,
        rows: MENU_ROWS,
        title_key: TITLE_KEY,
    })
}

/// `playing` stays until a menu is requested; that request is a container screen.
pub fn show(screen: Screen, menu: Option<Menu>) -> Screen {
    if menu.is_some() {
        Screen::Container
    } else {
        screen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terminal() -> Part<'static> {
        Part {
            id: TERMINAL,
            grid_node: true,
        }
    }

    #[test]
    fn empty_hand_on_the_stored_terminal_leaves_playing() {
        let parts = [("north", terminal())];
        let menu = open(true, "north", &parts);
        assert_eq!(
            menu,
            Some(Menu {
                id: MENU_ID,
                rows: MENU_ROWS,
                title_key: TITLE_KEY,
            })
        );
        assert_eq!(show(Screen::Playing, menu).label(), "container");
        assert_eq!(parts, [("north", terminal())]);
    }

    #[test]
    fn a_click_that_does_not_open_stays_on_playing() {
        let stored = [("north", terminal())];
        let stayed = [
            open(false, "north", &stored),
            open(true, "north", &[]),
            open(true, "up", &stored),
            open(
                true,
                "north",
                &[(
                    "north",
                    Part {
                        id: TERMINAL,
                        grid_node: false,
                    },
                )],
            ),
            open(
                true,
                "north",
                &[(
                    "north",
                    Part {
                        id: "ae2:facade",
                        grid_node: true,
                    },
                )],
            ),
            open(
                true,
                "north",
                &[(
                    "north",
                    Part {
                        id: "ae2:crafting_terminal",
                        grid_node: true,
                    },
                )],
            ),
            open(true, "side", &[("side", terminal())]),
            open(
                true,
                "north",
                &[("north", terminal()), ("north", terminal())],
            ),
        ];
        for menu in stayed {
            assert_eq!(menu, None);
            assert_eq!(show(Screen::Playing, menu).label(), "playing");
        }
    }

    #[test]
    fn the_cable_center_does_not_add_a_part_or_block_the_terminal() {
        let parts = [
            (
                "cable",
                Part {
                    id: "ae2:fluix_glass_cable",
                    grid_node: true,
                },
            ),
            ("north", terminal()),
        ];
        let menu = open(true, "north", &parts);
        assert!(menu.is_some());
        assert_eq!(parts.len(), 2);
        assert_eq!(
            parts.iter().filter(|(_, part)| part.id == TERMINAL).count(),
            1
        );
    }
}
