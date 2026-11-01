// SPDX-License-Identifier: LGPL-3.0-or-later
// CablePart/DenseCablePart shape and visual rules, AECableType/AEColor API data.
// API-derived portions retain the notice in LICENSES/AE2-API-MIT.
use super::ae2_grid::{ChannelMode, Direction, Flags};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    White,
    LightGray,
    Gray,
    Black,
    Lime,
    Yellow,
    Orange,
    Brown,
    Red,
    Pink,
    Magenta,
    Purple,
    Blue,
    LightBlue,
    Cyan,
    Green,
    Transparent,
}
impl Color {
    pub const ALL: [Self; 17] = [
        Self::White,
        Self::LightGray,
        Self::Gray,
        Self::Black,
        Self::Lime,
        Self::Yellow,
        Self::Orange,
        Self::Brown,
        Self::Red,
        Self::Pink,
        Self::Magenta,
        Self::Purple,
        Self::Blue,
        Self::LightBlue,
        Self::Cyan,
        Self::Green,
        Self::Transparent,
    ];
    pub fn from_ordinal(value: usize) -> Option<Self> {
        Self::ALL.get(value).copied()
    }
    pub fn prefix(self) -> &'static str {
        [
            "white",
            "light_gray",
            "gray",
            "black",
            "lime",
            "yellow",
            "orange",
            "brown",
            "red",
            "pink",
            "magenta",
            "purple",
            "blue",
            "light_blue",
            "cyan",
            "green",
            "fluix",
        ][self as usize]
    }
    pub fn from_prefix(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.prefix() == value)
    }
    pub fn compatible(self, other: Self) -> bool {
        self == Self::Transparent || other == Self::Transparent || self == other
    }
    pub fn variants(self) -> [i32; 4] {
        [
            [0xb4b4b4, 0xe0e0e0, 0xf9f9f9, 0x000000],
            [0x7e7e7e, 0xa09fa0, 0xc4c4c4, 0x000000],
            [0x4f4f4f, 0x6c6b6c, 0x949294, 0x000000],
            [0x131313, 0x272727, 0x3b3b3b, 0xffffff],
            [0x4ec04e, 0x70e259, 0xb3f86d, 0x000000],
            [0xffcf40, 0xffe359, 0xf4ff80, 0x000000],
            [0xd9782f, 0xeca23c, 0xf2ba49, 0x000000],
            [0x6e4a12, 0x7e5c16, 0x8e6e1a, 0x000000],
            [0xaa212b, 0xd73e42, 0xf07665, 0x000000],
            [0xd86eaa, 0xff99bb, 0xfbcad5, 0x000000],
            [0xc15189, 0xd5719c, 0xe69ebf, 0x000000],
            [0x6e5cb8, 0x915dcd, 0xb06fdd, 0x000000],
            [0x337ff0, 0x3894ff, 0x40c1ff, 0x000000],
            [0x69b9ff, 0x70d2ff, 0x80f7ff, 0x000000],
            [0x22b0ae, 0x2fccb7, 0x65e8c9, 0x000000],
            [0x079b6b, 0x17b86d, 0x32d850, 0x000000],
            [0x5a479e, 0x915dcd, 0xe2a3e3, 0x000000],
        ][self as usize]
    }
    pub fn tint(self, index: i32) -> i32 {
        let [dark, medium, bright, _] = self.variants();
        match index {
            1 => dark,
            2 => medium,
            3 => bright,
            4 => {
                let mut result = 0;
                for shift in [16, 8, 0] {
                    result |= (((bright >> shift & 255) + (medium >> shift & 255)) / 2) << shift;
                }
                result
            }
            _ => -1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CableType {
    None,
    Glass,
    Covered,
    Smart,
    DenseCovered,
    DenseSmart,
}
impl CableType {
    pub const ALL: [Self; 6] = [
        Self::None,
        Self::Glass,
        Self::Covered,
        Self::Smart,
        Self::DenseCovered,
        Self::DenseSmart,
    ];
    pub const VALID: [Self; 5] = [
        Self::Glass,
        Self::Covered,
        Self::Smart,
        Self::DenseCovered,
        Self::DenseSmart,
    ];
    pub fn from_ordinal(value: usize) -> Option<Self> {
        Self::ALL.get(value).copied()
    }
    pub fn is_dense(self) -> bool {
        matches!(self, Self::DenseCovered | Self::DenseSmart)
    }
    pub fn is_smart(self) -> bool {
        matches!(self, Self::Smart | Self::DenseSmart)
    }
    pub fn variant(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Glass => 1,
            Self::Covered | Self::DenseCovered => 2,
            Self::Smart | Self::DenseSmart => 3,
        }
    }
    pub fn size(self) -> u8 {
        if self == Self::None {
            0
        } else if self.is_dense() {
            2
        } else {
            1
        }
    }
    fn from_variant_size(variant: u8, size: u8) -> Self {
        match (variant, size) {
            (1, 1) => Self::Glass,
            (2, 1) => Self::Covered,
            (3, 1) => Self::Smart,
            (2, 2) => Self::DenseCovered,
            (3, 2) => Self::DenseSmart,
            _ => Self::None,
        }
    }
    pub fn min(self, other: Self) -> Self {
        Self::from_variant_size(
            self.variant().min(other.variant()),
            self.size().min(other.size()),
        )
    }
    pub fn max(self, other: Self) -> Self {
        Self::from_variant_size(
            self.variant().max(other.variant()),
            self.size().max(other.size()),
        )
    }
    pub fn suffix(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Glass => Some("glass_cable"),
            Self::Covered => Some("covered_cable"),
            Self::Smart => Some("smart_cable"),
            Self::DenseCovered => Some("covered_dense_cable"),
            Self::DenseSmart => Some("smart_dense_cable"),
        }
    }
    pub fn node_flags(self) -> Flags {
        if self == Self::None {
            Flags::default()
        } else if self.is_dense() {
            Flags::DENSE_CAPACITY
        } else {
            Flags::PREFERRED
        }
    }
    pub fn connection_length(self, other: Self) -> f64 {
        if self == other {
            4.0
        } else if other as u8 >= self as u8 {
            -1.0
        } else {
            8.0
        }
    }
    pub fn visual_channels(
        self,
        channels: i32,
        grid_max: i32,
        mode: ChannelMode,
        node_present: bool,
    ) -> i8 {
        if !node_present {
            return 0;
        }
        let visual = if self == Self::None {
            0
        } else if self.is_dense() {
            32
        } else {
            8
        };
        if mode == ChannelMode::Infinite {
            return if channels <= 0 { 0 } else { visual as i8 };
        }
        if visual == 0 || grid_max == 0 {
            return 0;
        }
        let result = visual.min(channels.wrapping_mul(visual).wrapping_div(grid_max)) as i8;
        if result == 0 && channels > 0 {
            1
        } else {
            result
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cable {
    pub kind: CableType,
    pub color: Color,
}
impl Cable {
    pub fn from_item(id: &str) -> Option<Self> {
        let id = id.strip_prefix("ae2:")?;
        CableType::VALID.into_iter().find_map(|kind| {
            let prefix = id.strip_suffix(kind.suffix()?)?.strip_suffix('_')?;
            Some(Self {
                kind,
                color: Color::from_prefix(prefix)?,
            })
        })
    }
    pub fn item(self) -> Option<String> {
        Some(format!(
            "ae2:{}_{}",
            self.color.prefix(),
            self.kind.suffix()?
        ))
    }
}

/// Source addBox arguments in model pixels; no silent clamping/normalization.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Box3(pub [f64; 6]);
fn connection_box(direction: Direction, min: f64, max: f64, distance: f64) -> Box3 {
    Box3(match direction {
        Direction::Down => [min, distance, min, max, min, max],
        Direction::Up => [min, max, min, max, 16.0 - distance, max],
        Direction::North => [min, min, distance, max, max, min],
        Direction::South => [min, min, max, max, max, 16.0 - distance],
        Direction::West => [distance, min, min, min, max, max],
        Direction::East => [max, min, min, 16.0 - distance, max, max],
    })
}
/// getBoxes' connection filter has a seventh bit for the null/center side.
/// Dense neighbour types and side-part connection lengths come from the host.
pub fn boxes(
    kind: CableType,
    connections: u8,
    dense_neighbours: u8,
    bb_collision: bool,
    filter: u8,
    part_lengths: [Option<f64>; 6],
) -> Vec<Box3> {
    if kind == CableType::None {
        return Vec::new();
    }
    let (min, max) = if kind.is_dense() {
        if bb_collision {
            (4.9, 11.1)
        } else {
            (3.0, 13.0)
        }
    } else if kind == CableType::Glass {
        (6.0, 10.0)
    } else {
        (5.0, 11.0)
    };
    let mut result = Vec::new();
    if filter & 64 != 0 {
        result.push(Box3([min, min, min, max, max, max]));
    }
    if !kind.is_dense() {
        for (i, direction) in Direction::ALL.into_iter().enumerate() {
            if filter & (1 << i) != 0
                && let Some(distance) = part_lengths[i]
            {
                if distance <= 0.0 || distance > 8.0 {
                    continue;
                }
                result.push(connection_box(direction, min, max, distance));
            }
        }
    }
    for (i, direction) in Direction::ALL.into_iter().enumerate() {
        if connections & filter & (1 << i) == 0 {
            continue;
        }
        let (low, high) = if kind.is_dense() && dense_neighbours & (1 << i) == 0 {
            (5.0, 11.0)
        } else {
            (min, max)
        };
        result.push(connection_box(direction, low, high, 0.0));
    }
    result
}

/// Host input for CablePart's channel payload. Part channel counts are the
/// maximum over that part's grid connections, or zero for a missing part node.
#[derive(Clone, Copy, Debug)]
pub struct VisualInput {
    pub kind: CableType,
    pub connections: u8,
    pub part_mask: u8,
    pub part_channels: [i32; 6],
    pub external_channels: [i32; 6],
    pub grid_max: i32,
    pub mode: ChannelMode,
    pub node_present: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisualState {
    pub connections: u8,
    pub channels: [i32; 6],
}
impl VisualState {
    pub fn channels_on_side(&self, side: Direction, powered: bool) -> i32 {
        if powered {
            self.channels[side as usize]
        } else {
            0
        }
    }
    /// CablePart payload AFTER AEBasePart's prefix, not the Minecraft packet.
    pub fn encode(input: VisualInput) -> Vec<u8> {
        let VisualInput {
            kind,
            connections,
            part_mask,
            part_channels,
            external_channels,
            grid_max,
            mode,
            node_present,
        } = input;
        let connected = if node_present { connections & 63 } else { 0 };
        let mut bytes = vec![connected];
        for i in 0..6 {
            if (connected | part_mask) & (1 << i) == 0 {
                continue;
            }
            let used = if connected & (1 << i) != 0 {
                external_channels[i]
            } else {
                part_channels[i]
            };
            // A side attachment has a node of its own; a missing center node
            // still forces getVisualChannels to zero, as in CablePart.
            bytes.push(kind.visual_channels(used, grid_max, mode, node_present) as u8);
        }
        bytes
    }
    pub fn decode(
        &mut self,
        bytes: &[u8],
        part_mask: u8,
        base_changed: bool,
    ) -> Result<(bool, usize), &'static str> {
        let connected = *bytes.first().ok_or("truncated cable connection byte")? & 63;
        let needed = 1 + ((connected | part_mask) & 63).count_ones() as usize;
        if bytes.len() < needed {
            return Err("truncated cable channel bytes");
        }
        let mut channels = [0; 6];
        let mut cursor = 1;
        for (i, channel) in channels.iter_mut().enumerate() {
            if (connected | part_mask) & (1 << i) != 0 {
                *channel = bytes[cursor] as i32;
                cursor += 1;
            }
        }
        let changed = base_changed || connected != self.connections || channels != self.channels;
        self.connections = connected;
        self.channels = channels;
        Ok((changed, cursor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_eighty_five_source_item_ids_roundtrip_and_unlike_colours_need_fluix() {
        for kind in CableType::VALID {
            for color in Color::ALL {
                let cable = Cable { kind, color };
                assert_eq!(Cable::from_item(&cable.item().unwrap()), Some(cable));
            }
        }
        assert_eq!(Cable::from_item("ae2:transparent_glass_cable"), None);
        assert_eq!(Cable::from_item("other:fluix_glass_cable"), None);
        for a in Color::ALL {
            for b in Color::ALL {
                assert_eq!(
                    a.compatible(b),
                    a == b || a == Color::Transparent || b == Color::Transparent
                );
            }
        }
    }
    #[test]
    fn variant_and_size_combine_independently_and_dense_flags_replace_preferred() {
        assert_eq!(
            CableType::DenseCovered.min(CableType::Smart),
            CableType::Covered
        );
        assert_eq!(
            CableType::DenseCovered.max(CableType::Smart),
            CableType::DenseSmart
        );
        assert_eq!(
            CableType::Glass.connection_length(CableType::DenseCovered),
            -1.0
        );
        assert_eq!(
            CableType::DenseSmart.connection_length(CableType::Glass),
            8.0
        );
        assert!(!CableType::DenseSmart.node_flags().has(Flags::PREFERRED));
        assert!(CableType::Glass.node_flags().has(Flags::PREFERRED));
    }
    #[test]
    fn dense_selection_and_collision_boxes_keep_source_widths() {
        assert_eq!(
            boxes(CableType::DenseCovered, 1, 1, false, 127, [None; 6]),
            vec![
                Box3([3., 3., 3., 13., 13., 13.]),
                Box3([3., 0., 3., 13., 3., 13.])
            ]
        );
        assert_eq!(
            boxes(CableType::DenseCovered, 1, 0, true, 127, [None; 6]),
            vec![
                Box3([4.9, 4.9, 4.9, 11.1, 11.1, 11.1]),
                Box3([5., 0., 5., 11., 5., 11.])
            ]
        );
    }
    #[test]
    fn visual_scaling_and_packet_reader_reset_unused_sides_without_touching_truncated_state() {
        assert_eq!(
            CableType::Smart.visual_channels(1, 32, ChannelMode::X4, true),
            1
        );
        assert_eq!(
            CableType::DenseSmart.visual_channels(32, 128, ChannelMode::X4, true),
            8
        );
        assert_eq!(
            CableType::Smart.visual_channels(1, 0, ChannelMode::Infinite, true),
            8
        );
        let bytes = VisualState::encode(VisualInput {
            kind: CableType::Smart,
            connections: 1,
            part_mask: 2,
            part_channels: [8; 6],
            external_channels: [1; 6],
            grid_max: 8,
            mode: ChannelMode::Default,
            node_present: true,
        });
        assert_eq!(bytes, vec![1, 1, 8]);
        let mut state = VisualState::default();
        assert_eq!(state.decode(&bytes, 2, false), Ok((true, 3)));
        assert_eq!(state.decode(&bytes, 2, false), Ok((false, 3)));
        let before = state.clone();
        assert!(state.decode(&[1], 2, false).is_err());
        assert_eq!(state, before);
        assert_eq!(state.decode(&[0], 0, false), Ok((true, 1)));
        assert_eq!(state.channels, [0; 6]);
    }
}
