// SPDX-License-Identifier: LGPL-3.0-or-later
// Numeric protocol shared with tools/pathing_oracle.py's original-Java harness.
use ae2_horizon::ae2_grid::{ChannelMode, Direction, Flags, Graph, NodeSpec};
use std::io::{self, Read};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut tokens = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let cases = tokens.next().unwrap();
    for _ in 0..cases {
        let mode = match tokens.next().unwrap() {
            0 => ChannelMode::Infinite,
            1 => ChannelMode::Default,
            2 => ChannelMode::X2,
            3 => ChannelMode::X3,
            4 => ChannelMode::X4,
            _ => panic!(),
        };
        let n = tokens.next().unwrap() as usize;
        let m = tokens.next().unwrap() as usize;
        let mut g = Graph::default();
        for _ in 0..n {
            let flags = Flags::from_bits(tokens.next().unwrap() as u8).unwrap();
            let controller = tokens.next().unwrap() != 0;
            let pos = std::array::from_fn(|_| tokens.next().unwrap() as i32);
            let count = tokens.next().unwrap();
            let multiblock = (0..count).map(|_| tokens.next().unwrap() as u64).collect();
            g.add_node(NodeSpec {
                flags,
                controller: controller.then_some(pos),
                multiblock,
            });
        }
        let mut edges = vec![];
        for _ in 0..m {
            let a = tokens.next().unwrap() as u64;
            let b = tokens.next().unwrap() as u64;
            let d = tokens.next().unwrap();
            edges.push(
                g.connect(
                    a,
                    b,
                    if d < 0 {
                        None
                    } else {
                        Some(Direction::ALL[d as usize])
                    },
                )
                .unwrap(),
            );
        }
        let reports = g.calculate(mode);
        assert_eq!(reports.len(), 1, "oracle input must be connected");
        let r = &reports[0];
        let mut output = vec![
            r.channels_in_use.to_string(),
            r.channels_by_blocks.to_string(),
        ];
        for id in 0..n {
            output.push(g.node(id as u64).unwrap().used_channels().to_string());
        }
        for id in edges {
            output.push(g.connection(id).unwrap().used_channels().to_string());
        }
        println!("{}", output.join(" "));
    }
}
