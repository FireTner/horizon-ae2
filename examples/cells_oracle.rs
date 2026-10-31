use ae2_horizon::{
    ae2_backend::CellBackend,
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
    tool: bool,
}
impl CellKey for Key {
    fn key_type(&self) -> KeyType {
        self.kind
    }
    fn can_fit_inside_cell(&self) -> bool {
        self.kind == KeyType::Fluid || self.id != 79
    }
    fn blacklisted(&self) -> bool {
        !self.tool && self.id == 78
    }
    fn primary_id(&self) -> Option<&str> {
        Some(if self.id.is_multiple_of(7) {
            "rejected"
        } else {
            "allowed"
        })
    }
}
fn key(kind: KeyType, id: usize, tool: bool) -> Key {
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
        tool,
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
        let profile = input.next().unwrap();
        let tool = profile >= 2;
        let layout = match profile {
            0 => CellLayout::basic(tier, kind),
            1 => CellLayout::portable(tier, kind),
            2 => CellLayout::matter_cannon(),
            3 => CellLayout::color_applicator(),
            _ => panic!("unknown cell profile"),
        };
        let flags = input.next().unwrap();
        let filters = input.next().unwrap();
        let partition: BTreeSet<_> = (0..filters)
            .map(|_| key(kind, input.next().unwrap() as usize, tool))
            .collect();
        let wrap = |mut cell: Cell<Key>| {
            cell.configure(
                partition.clone(),
                flags & 1 != 0,
                flags & 2 != 0,
                flags & 4 != 0,
            );
            if tool {
                CellBackend::restricted(cell, BTreeSet::from(["allowed".to_owned()]))
            } else {
                cell.into()
            }
        };
        let mut backend = wrap(Cell::new_with_layout(tier, kind, layout));
        let operations = input.next().unwrap();
        for _ in 0..operations {
            let op = input.next().unwrap();
            let id = input.next().unwrap() as usize;
            let amount = input.next().unwrap();
            let commit = input.next().unwrap() != 0;
            let result = match op {
                0 => backend.insert(key(kind, id, tool), amount, commit),
                1 => backend.extract(&key(kind, id, tool), amount, commit),
                2 => {
                    let contents = backend
                        .stacks()
                        .iter()
                        .filter(|(_, n)| **n > 0)
                        .map(|(k, n)| (k.clone(), *n))
                        .collect();
                    backend =
                        wrap(Cell::restore_with_layout(tier, kind, layout, contents).unwrap());
                    0
                }
                _ => panic!("unknown operation"),
            };
            let cell = backend.finite().unwrap();
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
                    cell.stacks()
                        .get(&key(kind, id, tool))
                        .copied()
                        .unwrap_or(0)
                );
            }
            println!();
        }
    }
    assert!(input.next().is_none());
}
