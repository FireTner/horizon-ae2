use ae2_horizon::ae2_cable::{Cable, CableType, Color};
use ae2_horizon::ae2_cable_render::{self as render, Core, Piece, RenderState};
use ae2_horizon::ae2_cube_builder::{CubeBuilder, Quad, Texture};
use ae2_horizon::ae2_grid::Direction;
use std::io::{self, BufRead, BufWriter, Write};
fn n<T: std::str::FromStr>(s: &mut std::str::SplitWhitespace<'_>) -> T {
    s.next().unwrap().parse().ok().unwrap()
}
fn f(s: &mut std::str::SplitWhitespace<'_>) -> f32 {
    f32::from_bits(u32::from_str_radix(s.next().unwrap(), 16).unwrap())
}
fn rect(s: &mut std::str::SplitWhitespace<'_>) -> [f32; 4] {
    std::array::from_fn(|_| f(s))
}
fn core(s: &mut std::str::SplitWhitespace<'_>) -> Core {
    [Core::Glass, Core::Covered, Core::Dense][n::<usize>(s)]
}
fn cable(s: &mut std::str::SplitWhitespace<'_>) -> Cable {
    Cable {
        kind: CableType::from_ordinal(n(s)).unwrap(),
        color: Color::from_ordinal(n(s)).unwrap(),
    }
}
fn print(out: &mut impl Write, quads: Vec<Quad>) {
    write!(out, "{}", quads.len()).unwrap();
    for q in quads {
        write!(
            out,
            " {} {} {} {:x} {}",
            q.texture,
            q.face as u8,
            q.cull.map_or(-1, |d| d as i32),
            q.color,
            q.emissive as u8
        )
        .unwrap();
        for i in 0..4 {
            for v in q.positions[i].into_iter().chain(q.uv[i]) {
                let bits = if v.is_nan() { 0x7fc00000 } else { v.to_bits() };
                write!(out, " {bits:x}").unwrap();
            }
        }
    }
    writeln!(out).unwrap();
}
fn main() {
    let mut out = BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut s = line.split_whitespace();
        let quads = match s.next().unwrap() {
            "C" => {
                let faces = n(&mut s);
                let fu = n::<u8>(&mut s);
                let fv = n::<u8>(&mut s);
                let rot = n::<u16>(&mut s);
                let custom = n::<u8>(&mut s);
                let bright = n::<u8>(&mut s) != 0;
                let color = u32::from_str_radix(s.next().unwrap(), 16).unwrap();
                let mode = n::<u8>(&mut s);
                let face = Direction::ALL[n::<usize>(&mut s)];
                let bounds = std::array::from_fn(|_| f(&mut s));
                let uv = rect(&mut s);
                let atlas = rect(&mut s);
                let mut b = CubeBuilder::default();
                b.draw_faces = faces;
                b.color = color;
                b.emissive = bright;
                for (i, face) in Direction::ALL.into_iter().enumerate() {
                    b.set_face_texture(
                        face,
                        Texture {
                            id: format!("probe/{i}"),
                            rect: atlas,
                        },
                    );
                    b.set_flip_u(face, fu & (1 << i) != 0);
                    b.set_flip_v(face, fv & (1 << i) != 0);
                    b.set_rotation(face, usize::from((rot >> (i * 2)) & 3))
                        .unwrap();
                    if custom & (1 << i) != 0 {
                        b.set_custom_uv(face, uv);
                    }
                }
                if mode == 0 {
                    b.add_cube(bounds);
                } else {
                    b.add_quad(face, bounds);
                }
                b.output
            }
            "P" => {
                let cable = cable(&mut s);
                let face = Direction::ALL[n::<usize>(&mut s)];
                let op = n::<u8>(&mut s);
                let other = CableType::from_ordinal(n(&mut s)).unwrap();
                let adjacent = n::<u8>(&mut s) != 0;
                let channels = n(&mut s);
                let distance = n(&mut s);
                let core = core(&mut s);
                let atlas = rect(&mut s);
                let piece = match op {
                    0 => Piece::Core(core),
                    1 => Piece::Connection {
                        other,
                        adjacent,
                        channels,
                    },
                    2 => Piece::Straight { channels },
                    3 => Piece::Constrained { distance, channels },
                    _ => unreachable!(),
                };
                render::piece(cable, face, piece, &|id| Texture {
                    id: id.into(),
                    rect: atlas,
                })
            }
            "R" => {
                let cable = cable(&mut s);
                let core = core(&mut s);
                let require = n::<u8>(&mut s) != 0;
                let adjacent = n(&mut s);
                let channels = std::array::from_fn(|_| n(&mut s));
                let connections = std::array::from_fn(|_| {
                    let value = n::<i32>(&mut s);
                    if value < 0 {
                        None
                    } else {
                        Some(CableType::from_ordinal(value as usize).unwrap())
                    }
                });
                let attachments = std::array::from_fn(|_| {
                    let value = n::<i32>(&mut s);
                    (value != i32::MIN).then_some(value)
                });
                let atlas = rect(&mut s);
                render::render(
                    &RenderState {
                        cable,
                        core,
                        connections,
                        adjacent,
                        channels,
                        attachments,
                        attachment_requires_connection: require,
                    },
                    &|id| Texture {
                        id: id.into(),
                        rect: atlas,
                    },
                )
            }
            _ => unreachable!(),
        };
        assert!(s.next().is_none());
        print(&mut out, quads);
    }
}
