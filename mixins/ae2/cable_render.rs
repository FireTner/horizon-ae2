// SPDX-License-Identifier: LGPL-3.0-or-later
// CableBuilder/SmartCableTextures/CableBusBakedModel cable-only geometry rules.
use super::ae2_cable::{Cable, CableType, Color};
use super::ae2_cube_builder::{CubeBuilder, Quad, Texture};
use super::ae2_grid::Direction;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Core {
    Glass,
    Covered,
    Dense,
}
impl Core {
    pub fn of(kind: CableType) -> Option<Self> {
        match kind {
            CableType::None => None,
            CableType::Glass => Some(Self::Glass),
            CableType::Covered | CableType::Smart => Some(Self::Covered),
            _ => Some(Self::Dense),
        }
    }
    fn folder(self) -> &'static str {
        match self {
            Self::Glass => "glass",
            Self::Covered => "covered",
            Self::Dense => "dense_smart",
        }
    }
}
pub fn texture_color(color: Color) -> &'static str {
    if color == Color::Transparent {
        "transparent"
    } else {
        color.prefix()
    }
}
fn folder(kind: CableType) -> &'static str {
    match kind {
        CableType::None => panic!("no connection texture"),
        CableType::Glass => "glass",
        CableType::Covered => "covered",
        CableType::Smart => "smart",
        CableType::DenseCovered => "dense_covered",
        CableType::DenseSmart => "dense_smart",
    }
}
pub fn connection_texture(kind: CableType, color: Color) -> String {
    format!("ae2:part/cable/{}/{}", folder(kind), texture_color(color))
}
pub fn core_texture(core: Core, color: Color) -> String {
    format!(
        "ae2:part/cable/core/{}/{}",
        core.folder(),
        texture_color(color)
    )
}
pub fn channel_texture(dense: bool, even: bool, channels: i32) -> String {
    let n = if even {
        if channels < 5 {
            0
        } else if channels <= 8 {
            channels - 4
        } else {
            4
        }
    } else {
        channels.clamp(0, 4)
    };
    format!(
        "ae2:part/cable/{}/channels_{}{}",
        if dense { "dense_smart" } else { "smart" },
        even as u8,
        n
    )
}
/// Each piece maps directly to a CableBuilder public operation.
#[derive(Clone, Copy, Debug)]
pub enum Piece {
    Core(Core),
    Connection {
        other: CableType,
        adjacent: bool,
        channels: i32,
    },
    Straight {
        channels: i32,
    },
    Constrained {
        distance: i32,
        channels: i32,
    },
}
#[derive(Clone, Copy)]
enum Size {
    Glass,
    Covered,
    Dense,
    Big,
}
fn sized(b: &mut CubeBuilder, face: Direction, size: Size, distance: i32) {
    let [min, max, end] = match size {
        Size::Glass => [6., 10., 6.],
        Size::Covered => [6., 10., 5.],
        Size::Dense => [4., 12., 5.],
        Size::Big => [5., 11., 4.],
    };
    let dist = distance as f32;
    // Java computes 16 - distanceFromEdge as int before its float conversion.
    let opposite = 16_i32.wrapping_sub(distance) as f32;
    let bounds = match face {
        Direction::Down => [min, dist, min, max, end, max],
        Direction::Up => [min, 16. - end, min, max, opposite, max],
        Direction::North => [min, min, dist, max, max, end],
        Direction::South => [min, min, 16. - end, max, max, opposite],
        Direction::West => [dist, min, min, end, max, max],
        Direction::East => [16. - end, min, min, opposite, max, max],
    };
    b.add_cube(bounds);
}
fn straight_uv(b: &mut CubeBuilder, face: Direction, x: f32, y: f32) {
    use Direction::*;
    let rows: &[(Direction, [f32; 4])] = match face {
        Down | Up => &[
            (North, [x, 0., y, x]),
            (East, [x, 0., y, x]),
            (South, [x, 0., y, x]),
            (West, [x, 0., y, x]),
        ],
        East | West => &[
            (Up, [0., x, x, y]),
            (Down, [0., x, x, y]),
            (North, [0., x, x, y]),
            (South, [0., x, x, y]),
        ],
        North | South => &[
            (Up, [x, 0., y, x]),
            (Down, [x, 0., y, x]),
            (East, [0., x, x, y]),
            (West, [0., x, x, y]),
        ],
    };
    for (f, uv) in rows {
        b.set_custom_uv(*f, *uv);
    }
}
fn straight_cube(b: &mut CubeBuilder, face: Direction, dense: bool) {
    use Direction::*;
    let (min, max, lo, hi) = if dense {
        (3., 13., -0.01, 16.01)
    } else {
        (5., 11., 0., 16.)
    };
    let (rotations, bounds): (&[Direction], [f32; 6]) = match face {
        Up | Down => (&[East], [min, lo, min, max, hi, max]),
        East | West => (&[South, North], [lo, min, min, hi, max, max]),
        North | South => (&[East, West], [min, min, lo, max, max, hi]),
    };
    for d in rotations {
        b.set_rotation(*d, 2).unwrap();
    }
    b.add_cube(bounds);
    for d in rotations {
        b.set_rotation(*d, 0).unwrap();
    }
}
fn flip_connection(b: &mut CubeBuilder, face: Direction, dense: bool) {
    use Direction::*;
    if dense {
        match face {
            West | East => b.set_flip_v(Down, true),
            Up | Down => {
                b.set_flip_u(North, true);
                b.set_flip_u(East, true);
            }
            _ => {}
        }
    } else {
        match face {
            Down => {
                b.set_flip_u(East, true);
                b.set_flip_u(North, true);
            }
            Up => {
                b.set_flip_u(East, true);
                b.set_flip_u(North, true);
                b.set_flip_v(Down, true);
            }
            South => b.set_flip_u(North, true),
            West => {
                b.set_flip_v(Down, true);
                b.set_flip_u(East, true);
            }
            East => b.set_flip_v(Down, true),
            North => {}
        }
    }
}
fn layered(
    b: &mut CubeBuilder,
    cable: Cable,
    channels: i32,
    dense: bool,
    lookup: &impl Fn(&str) -> Texture,
    mut shape: impl FnMut(&mut CubeBuilder),
) {
    shape(b);
    b.emissive = true;
    b.set_texture(lookup(&channel_texture(dense, false, channels)));
    b.set_rgb(cable.color.variants()[0] as u32);
    shape(b);
    b.set_texture(lookup(&channel_texture(dense, true, channels)));
    b.set_rgb(cable.color.variants()[2] as u32);
    shape(b);
    b.emissive = false;
}
pub fn piece(
    cable: Cable,
    face: Direction,
    piece: Piece,
    lookup: &impl Fn(&str) -> Texture,
) -> Vec<Quad> {
    use CableType::*;
    use Direction::*;
    if cable.kind == None && !matches!(piece, Piece::Core(_)) {
        return vec![];
    }
    let mut b = CubeBuilder::default();
    match piece {
        Piece::Core(core) => {
            b.set_texture(lookup(&core_texture(core, cable.color)));
            let (lo, hi) = match core {
                Core::Glass => (6., 10.),
                Core::Covered => (5., 11.),
                Core::Dense => (3., 13.),
            };
            b.add_cube([lo, lo, lo, hi, hi, hi]);
        }
        Piece::Connection {
            other,
            adjacent,
            channels,
        } => {
            let kind = match cable.kind {
                Smart if matches!(other, Glass | Covered) => Covered,
                DenseCovered if matches!(other, Glass | Covered | Smart) => Covered,
                DenseSmart => match other {
                    Glass | Covered => Covered,
                    Smart => Smart,
                    DenseCovered => DenseCovered,
                    _ => DenseSmart,
                },
                k => k,
            };
            b.draw_faces = 63 & !(1 << face as u8);
            b.set_texture(lookup(&connection_texture(kind, cable.color)));
            match kind {
                Glass => {
                    if other != Glass && !adjacent {
                        b.set_texture(lookup(&connection_texture(Covered, cable.color)));
                        sized(&mut b, face, Size::Big, 0);
                        b.set_texture(lookup(&connection_texture(Glass, cable.color)));
                    }
                    sized(&mut b, face, Size::Glass, 0);
                }
                Covered => {
                    if other != Glass && !adjacent {
                        sized(&mut b, face, Size::Big, 0);
                    }
                    sized(&mut b, face, Size::Covered, 0);
                }
                Smart => {
                    flip_connection(&mut b, face, false);
                    if other != Glass && !adjacent {
                        layered(&mut b, cable, channels, false, lookup, |b| {
                            sized(b, face, Size::Big, 0)
                        });
                        b.set_texture(lookup(&connection_texture(Smart, cable.color)));
                    }
                    // Source retains the last bright vertex color after resetting
                    // texture/fullbright for the small base segment.
                    layered(&mut b, cable, channels, false, lookup, |b| {
                        sized(b, face, Size::Covered, 0)
                    });
                }
                DenseCovered => sized(&mut b, face, Size::Dense, 0),
                DenseSmart => {
                    flip_connection(&mut b, face, true);
                    let channels = channels.wrapping_add(3) / 4;
                    layered(&mut b, cable, channels, true, lookup, |b| {
                        sized(b, face, Size::Dense, 0)
                    });
                }
                None => {}
            }
        }
        Piece::Straight { channels } => {
            b.set_texture(lookup(&connection_texture(cable.kind, cable.color)));
            if cable.kind == Glass {
                b.draw_faces = 63 & !(1 << face as u8) & !(1 << face.opposite() as u8);
                let bounds = match face {
                    Down | Up => [6., 0., 6., 10., 16., 10.],
                    North | South => [6., 6., 0., 10., 10., 16.],
                    West | East => [0., 6., 6., 16., 10., 10.],
                };
                b.add_cube(bounds);
            } else {
                let dense = cable.kind.is_dense();
                if cable.kind == Smart {
                    match face {
                        East | West => b.set_flip_v(Down, true),
                        Up | Down => b.set_flip_u(North, true),
                        _ => {}
                    }
                }
                if cable.kind == DenseSmart {
                    match face {
                        North => b.set_flip_u(North, true),
                        West => {
                            b.set_flip_v(Down, true);
                            b.set_flip_u(East, true);
                        }
                        Down => {
                            b.set_flip_u(North, true);
                            b.set_flip_v(Down, true);
                        }
                        _ => {}
                    }
                }
                let (x, y) = if dense {
                    (3. / 16., 13. / 16.)
                } else {
                    (5. / 16., 11. / 16.)
                };
                straight_uv(&mut b, face, x, y);
                if cable.kind.is_smart() {
                    let channels = if dense {
                        channels.wrapping_add(3) / 4
                    } else {
                        channels
                    };
                    layered(&mut b, cable, channels, dense, lookup, |b| {
                        straight_cube(b, face, dense)
                    });
                } else {
                    straight_cube(&mut b, face, dense);
                }
            }
        }
        Piece::Constrained { distance, channels } => {
            if cable.kind.is_dense() {
                return vec![];
            }
            let max = if cable.kind == Glass { 6 } else { 5 };
            if distance >= max {
                return vec![];
            }
            b.set_texture(lookup(&connection_texture(cable.kind, cable.color)));
            let size = if cable.kind == Glass {
                Size::Glass
            } else {
                Size::Covered
            };
            if cable.kind == Smart {
                match face {
                    Up | Down => {
                        b.set_flip_u(East, true);
                        b.set_flip_u(North, true);
                    }
                    East | West => b.set_flip_v(Down, true),
                    _ => {}
                }
                layered(&mut b, cable, channels, false, lookup, |b| {
                    sized(b, face, size, distance)
                });
            } else {
                sized(&mut b, face, size, distance);
            }
        }
    }
    b.output
}
#[derive(Clone, Debug)]
pub struct RenderState {
    pub cable: Cable,
    pub core: Core,
    pub connections: [Option<CableType>; 6],
    pub adjacent: u8,
    pub channels: [i32; 6],
    pub attachments: [Option<i32>; 6],
    pub attachment_requires_connection: bool,
}
pub fn render(state: &RenderState, lookup: &impl Fn(&str) -> Texture) -> Vec<Quad> {
    if state.cable.kind == CableType::None {
        return vec![];
    }
    let sides: Vec<_> = Direction::ALL
        .into_iter()
        .filter(|d| state.connections[*d as usize].is_some())
        .collect();
    if !state.attachment_requires_connection
        && sides.len() == 2
        && sides[0].opposite() == sides[1]
        && sides
            .iter()
            .all(|d| state.connections[*d as usize] == Some(state.cable.kind))
    {
        return piece(
            state.cable,
            sides[0],
            Piece::Straight {
                channels: state.channels[sides[0] as usize],
            },
            lookup,
        );
    }
    let mut quads = piece(
        state.cable,
        Direction::Down,
        Piece::Core(state.core),
        lookup,
    );
    for (i, side) in Direction::ALL.into_iter().enumerate() {
        if let Some(distance) = state.attachments[i] {
            quads.extend(piece(
                state.cable,
                side,
                Piece::Constrained {
                    distance,
                    channels: state.channels[i],
                },
                lookup,
            ));
        }
    }
    for (i, side) in Direction::ALL.into_iter().enumerate() {
        if let Some(other) = state.connections[i] {
            quads.extend(piece(
                state.cable,
                side,
                Piece::Connection {
                    other,
                    adjacent: state.adjacent & (1 << i) != 0,
                    channels: state.channels[i],
                },
                lookup,
            ));
        }
    }
    quads
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fluix_texture_uses_transparent_and_smart_machine_cap_retains_source_vertex_color() {
        let cable = Cable {
            kind: CableType::Smart,
            color: Color::Red,
        };
        let q = piece(
            cable,
            Direction::Down,
            Piece::Connection {
                other: CableType::Smart,
                adjacent: false,
                channels: 3,
            },
            &|id| Texture::unit(id),
        );
        assert_eq!(q.len(), 30);
        assert_eq!(q[15].color, 0xfff07665);
        assert!(!q[15].emissive);
        assert_eq!(
            connection_texture(CableType::Glass, Color::Transparent),
            "ae2:part/cable/glass/transparent"
        );
        let q = piece(
            Cable {
                kind: CableType::DenseSmart,
                color: Color::Blue,
            },
            Direction::East,
            Piece::Straight { channels: 5 },
            &|id| Texture::unit(id),
        );
        assert_eq!(q.len(), 18);
        assert!(q[6].texture.ends_with("channels_02"));
        assert!(q[0].positions.iter().any(|p| p[0] > 1.));
    }
}
