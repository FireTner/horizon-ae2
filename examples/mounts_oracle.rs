use ae2_horizon::ae2_mounts::{
    Counts, Inventory, NetworkStorage, Storage, WeakNetworkStorage, add_count,
};
use std::io::{self, Read};
use std::sync::{Arc, Mutex};

struct Action {
    phase: u64,
    target: Inventory<u32, u64>,
    flags: u64,
    priority: i32,
}
struct Bin {
    count: u64,
    capacity: u64,
    preferred: bool,
    net: WeakNetworkStorage<u32, u64>,
    action: Option<Action>,
}
impl Bin {
    fn trigger(&mut self, phase: u64) {
        if self.action.as_ref().is_some_and(|a| a.phase == phase) {
            let a = self.action.take().unwrap();
            let net = self.net.upgrade().unwrap();
            if a.flags & 1 != 0 {
                net.unmount(&a.target);
            }
            if a.flags & 2 != 0 {
                net.mount(a.priority, a.target);
            }
            if a.flags & 4 != 0 {
                assert_eq!(net.insert(&1, 1, true, &42), 0);
            }
        }
    }
}
impl Storage<u32, u64> for Bin {
    fn preferred(&mut self, _: &u32, _: &u64) -> bool {
        self.trigger(1);
        self.preferred
    }
    fn insert(&mut self, _: &u32, amount: u64, commit: bool, _: &u64) -> u64 {
        self.trigger(2);
        let value = amount.min(self.capacity - self.count);
        if commit {
            self.count += value;
        }
        value
    }
    fn extract(&mut self, _: &u32, amount: u64, commit: bool, _: &u64) -> u64 {
        self.trigger(3);
        let value = amount.min(self.count);
        if commit {
            self.count -= value;
        }
        value
    }
    fn available(&mut self, out: &mut Counts<u32>) {
        self.trigger(4);
        add_count(out, 1, self.count as i64);
    }
}
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut tokens = input.split_whitespace();
    let mut next = || tokens.next().unwrap().parse::<i64>().unwrap();
    let cases = next();
    for _ in 0..cases {
        let n = next() as usize;
        let ops = next();
        let net = NetworkStorage::default();
        let mut bins = vec![];
        let mut inventories: Vec<Inventory<_, _>> = vec![];
        for _ in 0..n {
            let bin = Arc::new(Mutex::new(Bin {
                capacity: next() as u64,
                count: next() as u64,
                preferred: next() != 0,
                net: net.weak(),
                action: None,
            }));
            bins.push(bin.clone());
            inventories.push(bin);
        }
        inventories.push(Arc::new(Mutex::new(net.weak())));
        for _ in 0..ops {
            let op = next();
            let mut result = 0;
            match op {
                0 => {
                    let priority = next() as i32;
                    let id = next() as usize;
                    net.mount(priority, inventories[id].clone());
                }
                1 => {
                    net.unmount(&inventories[next() as usize]);
                }
                2 => {
                    result = net.insert(&1, next() as u64, next() != 0, &42) as i64;
                }
                3 => {
                    result = net.extract(&1, next() as u64, next() != 0, &42) as i64;
                }
                4 => {
                    let mut counts = Counts::new();
                    net.available(&mut counts);
                    result = counts.get(&1).copied().unwrap_or(0);
                }
                5 => {
                    let id = next() as usize;
                    let value = next() != 0;
                    bins[id].lock().unwrap().preferred = value;
                }
                6 => {
                    let id = next() as usize;
                    let phase = next() as u64;
                    let target = next() as usize;
                    let flags = next() as u64;
                    let priority = next() as i32;
                    bins[id].lock().unwrap().action = Some(Action {
                        phase,
                        target: inventories[target].clone(),
                        flags,
                        priority,
                    });
                }
                7 => {
                    let id = next() as usize;
                    let value = next() as u64;
                    bins[id].lock().unwrap().count = value;
                }
                _ => panic!("unknown operation"),
            }
            print!("{result}");
            for bin in &bins {
                print!(" {}", bin.lock().unwrap().count);
            }
            println!();
        }
        // Pending callback target handles may form provider cycles; drop them.
        for bin in &bins {
            bin.lock().unwrap().action = None;
        }
    }
}
