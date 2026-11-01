use ae2_horizon::ae2_cable::{CableType, Color, VisualInput, VisualState, boxes};
use ae2_horizon::ae2_grid::{ChannelMode, Direction};
use ae2_horizon::ae2_part_storage::PartStorage;
use std::io::{self, BufRead, BufWriter, Write};

fn next<T: std::str::FromStr>(tokens: &mut std::str::SplitWhitespace<'_>) -> T {
    tokens.next().unwrap().parse().ok().unwrap()
}
fn kind(tokens: &mut std::str::SplitWhitespace<'_>) -> CableType {
    CableType::from_ordinal(next(tokens)).unwrap()
}
fn color(tokens: &mut std::str::SplitWhitespace<'_>) -> Color {
    Color::from_ordinal(next(tokens)).unwrap()
}
fn mode(tokens: &mut std::str::SplitWhitespace<'_>) -> ChannelMode {
    if next::<u8>(tokens) == 0 {
        ChannelMode::Default
    } else {
        ChannelMode::Infinite
    }
}
fn main() {
    let mut out = BufWriter::new(io::stdout().lock());
    let mut storage: PartStorage<i32, i32, i32> = PartStorage::default();
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut t = line.split_whitespace();
        match t.next().unwrap() {
            "T" => {
                let a = kind(&mut t);
                let b = kind(&mut t);
                writeln!(
                    out,
                    "{} {} {} {} {} {} {} {}",
                    a.min(b) as u8,
                    a.max(b) as u8,
                    a.is_dense() as u8,
                    a.is_smart() as u8,
                    (a != CableType::None) as u8,
                    a.variant(),
                    a.size(),
                    a.connection_length(b) as i32
                )
                .unwrap();
            }
            "C" => {
                let a = color(&mut t);
                let tint = next(&mut t);
                let b = color(&mut t);
                let [dark, medium, bright, contrast] = a.variants();
                writeln!(
                    out,
                    "{} {dark} {medium} {bright} {contrast} {} {}",
                    a.prefix(),
                    a.tint(tint),
                    a.compatible(b) as u8
                )
                .unwrap();
            }
            "V" => {
                let a = kind(&mut t);
                let channels = next(&mut t);
                let max = next(&mut t);
                let mode = mode(&mut t);
                let present = next::<u8>(&mut t) != 0;
                writeln!(out, "{}", a.visual_channels(channels, max, mode, present)).unwrap();
            }
            "B" => {
                let a = kind(&mut t);
                let connections = next(&mut t);
                let dense = next(&mut t);
                let bb = next::<u8>(&mut t) != 0;
                let filter = next(&mut t);
                let lengths = std::array::from_fn(|_| {
                    let value = t.next().unwrap();
                    if value == "none" {
                        None
                    } else {
                        Some(value.parse::<f64>().unwrap())
                    }
                });
                let boxes = boxes(a, connections, dense, bb, filter, lengths);
                write!(out, "{}", boxes.len()).unwrap();
                for b in boxes {
                    for v in b.0 {
                        let bits = if v.is_nan() {
                            f64::NAN.to_bits()
                        } else {
                            v.to_bits()
                        };
                        write!(out, " {bits:x}").unwrap();
                    }
                }
                writeln!(out).unwrap();
            }
            "P" => {
                let kind = kind(&mut t);
                let connections = next(&mut t);
                let part_mask = next(&mut t);
                let grid_max = next(&mut t);
                let mode = mode(&mut t);
                let node_present = next::<u8>(&mut t) != 0;
                let part_channels = std::array::from_fn(|_| next(&mut t));
                let external_channels = std::array::from_fn(|_| next(&mut t));
                let mut state = VisualState {
                    connections: next(&mut t),
                    channels: std::array::from_fn(|_| next(&mut t)),
                };
                let base_changed = next::<u8>(&mut t) != 0;
                let bytes = VisualState::encode(VisualInput {
                    kind,
                    connections,
                    part_mask,
                    part_channels,
                    external_channels,
                    grid_max,
                    mode,
                    node_present,
                });
                let (changed, used) = state.decode(&bytes, part_mask, base_changed).unwrap();
                assert_eq!(used, bytes.len());
                for b in bytes {
                    write!(out, "{b:02x}").unwrap();
                }
                write!(out, " {} {}", changed as u8, state.connections).unwrap();
                for ch in state.channels {
                    write!(out, " {ch}").unwrap();
                }
                writeln!(out, " {}", state.channels_on_side(Direction::Down, false)).unwrap();
            }
            "S" => {
                let action = next::<u8>(&mut t);
                let side = Direction::ALL[next::<usize>(&mut t)];
                let value = next::<i32>(&mut t);
                let value = (value != 0).then_some(value);
                match action {
                    0 => storage = PartStorage::default(),
                    1 => storage.set_center(value),
                    2 => storage.set_part(side, value),
                    3 => storage.remove_part(side),
                    4 => storage.set_facade(side, value),
                    5 => storage.remove_facade(side),
                    _ => unreachable!(),
                }
                write!(
                    out,
                    "{} {} {}",
                    storage.parts_allocated() as u8,
                    storage.facades_allocated() as u8,
                    storage.center().copied().unwrap_or(0)
                )
                .unwrap();
                for side in Direction::ALL {
                    write!(out, " {}", storage.part(side).copied().unwrap_or(0)).unwrap();
                }
                for side in Direction::ALL {
                    write!(out, " {}", storage.facade(side).copied().unwrap_or(0)).unwrap();
                }
                writeln!(out).unwrap();
            }
            _ => unreachable!(),
        }
        assert!(t.next().is_none());
    }
}
