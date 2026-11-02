use ae2_horizon::{
    ae2_grid::Direction,
    ae2_part_placement::{CenterSlot, center_target},
};
use std::io::{self, BufRead};
fn slot(n: i32) -> CenterSlot {
    match n {
        0 => CenterSlot::NoHost {
            replaceable: false,
            may_interact: true,
        },
        1 => CenterSlot::NoHost {
            replaceable: true,
            may_interact: true,
        },
        2 => CenterSlot::NoHost {
            replaceable: true,
            may_interact: false,
        },
        3 => CenterSlot::Host {
            occupied: false,
            compatible: true,
        },
        4 => CenterSlot::Host {
            occupied: true,
            compatible: true,
        },
        5 => CenterSlot::Host {
            occupied: false,
            compatible: false,
        },
        _ => panic!("unknown fixture slot"),
    }
}
fn main() {
    for line in io::stdin().lock().lines() {
        let v: Vec<i32> = line
            .unwrap()
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let pos = [v[0], v[1], v[2]];
        let result = center_target(pos, Direction::ALL[v[3] as usize], v[4] == 1, |p| {
            slot(if p == pos { v[5] } else { v[6] })
        });
        if let Some(p) = result {
            println!("{} {} {} {}", p.pos[0], p.pos[1], p.pos[2], p.side as u8);
        } else {
            println!("-");
        }
    }
}
