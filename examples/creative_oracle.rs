use ae2_horizon::{ae2_backend::CreativeCell, ae2_keys::KeyType, ae2_storage::CellState};
use std::{
    collections::BTreeSet,
    io::{self, Read},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    kind: KeyType,
    id: usize,
}
fn key(kind: KeyType, id: usize) -> Key {
    Key {
        kind: if id == 81 {
            if kind == KeyType::Item {
                KeyType::Fluid
            } else {
                KeyType::Item
            }
        } else {
            kind
        },
        id,
    }
}
fn main() {
    let mut text = String::new();
    io::stdin().read_to_string(&mut text).unwrap();
    let mut input = text.split_whitespace().map(|s| s.parse::<u64>().unwrap());
    let cases = input.next().unwrap();
    for _ in 0..cases {
        let _tier = input.next().unwrap();
        let kind = if input.next().unwrap() == 0 {
            KeyType::Item
        } else {
            KeyType::Fluid
        };
        let _portable = input.next().unwrap();
        let _flags = input.next().unwrap();
        let filters = input.next().unwrap();
        let partition: BTreeSet<_> = (0..filters)
            .map(|_| key(kind, input.next().unwrap() as usize))
            .collect();
        let mut cell = CreativeCell::new(partition.clone());
        let operations = input.next().unwrap();
        for _ in 0..operations {
            let op = input.next().unwrap();
            let id = input.next().unwrap() as usize;
            let amount = input.next().unwrap();
            let commit = input.next().unwrap() != 0;
            let result = match op {
                0 => cell.insert(&key(kind, id), amount, commit),
                1 => cell.extract(&key(kind, id), amount, commit),
                2 => {
                    cell = CreativeCell::new(partition.clone());
                    0
                }
                _ => panic!("unknown operation"),
            };
            let state = match cell.state() {
                CellState::Empty => 1,
                CellState::NotEmpty => 2,
                CellState::TypesFull => 3,
                CellState::Full => 4,
            };
            print!(
                "{result} {state} {} {} {}",
                cell.idle_drain().to_bits(),
                u8::from(cell.can_fit_inside_cell()),
                u8::from(cell.preferred(&key(kind, id)))
            );
            for id in 0..82 {
                print!(
                    " {}",
                    cell.stacks().get(&key(kind, id)).copied().unwrap_or(0)
                );
            }
            println!();
        }
    }
    assert!(input.next().is_none());
}
