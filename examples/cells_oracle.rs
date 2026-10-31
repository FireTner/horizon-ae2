use ae2_horizon::{
    ae2_keys::KeyType,
    ae2_storage::{Cell, CellKey, CellLayout, CellState, Tier},
};
use std::{
    collections::BTreeSet,
    io::{self, Read},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    kind: KeyType,
    id: usize,
}
impl CellKey for Key {
    fn key_type(&self) -> KeyType {
        self.kind
    }
    fn can_fit_inside_cell(&self) -> bool {
        self.kind == KeyType::Fluid || self.id != 79
    }
    fn blacklisted(&self) -> bool {
        self.id == 78
    }
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
        let tier = Tier::ALL[input.next().unwrap() as usize];
        let kind = if input.next().unwrap() == 0 {
            KeyType::Item
        } else {
            KeyType::Fluid
        };
        let layout = if input.next().unwrap() == 0 {
            CellLayout::basic(tier, kind)
        } else {
            CellLayout::portable(tier, kind)
        };
        let flags = input.next().unwrap();
        let filters = input.next().unwrap();
        let partition: BTreeSet<_> = (0..filters)
            .map(|_| key(kind, input.next().unwrap() as usize))
            .collect();
        let mut cell = Cell::new_with_layout(tier, kind, layout);
        cell.configure(
            partition.clone(),
            flags & 1 != 0,
            flags & 2 != 0,
            flags & 4 != 0,
        );
        let operations = input.next().unwrap();
        for _ in 0..operations {
            let op = input.next().unwrap();
            let id = input.next().unwrap() as usize;
            let amount = input.next().unwrap();
            let commit = input.next().unwrap() != 0;
            let result = match op {
                0 => cell.insert(key(kind, id), amount, commit),
                1 => cell.extract(&key(kind, id), amount, commit),
                2 => {
                    let contents = cell
                        .stacks()
                        .iter()
                        .filter(|(_, n)| **n > 0)
                        .map(|(k, n)| (k.clone(), *n))
                        .collect();
                    cell = Cell::restore_with_layout(tier, kind, layout, contents).unwrap();
                    cell.configure(
                        partition.clone(),
                        flags & 1 != 0,
                        flags & 2 != 0,
                        flags & 4 != 0,
                    );
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
                "{result} {} {} {} {} {} {} {state}",
                cell.stored_count(),
                cell.stored_types(),
                cell.used_bytes(),
                cell.free_bytes(),
                cell.remaining_types(),
                cell.remaining_count()
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
