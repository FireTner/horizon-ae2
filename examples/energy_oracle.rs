use ae2_horizon::ae2_power::{EnergyNode, EnergyService, EnergySystem, Flow, Store};
use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut tokens = input.split_whitespace();
    let mut next = || tokens.next().unwrap().parse::<f64>().unwrap();
    let cases = next() as usize;
    for _ in 0..cases {
        let mut system = EnergySystem::default();
        system.add_service(0, EnergyService::default()).unwrap();
        let mut current = 0;
        let count = next() as usize;
        let mut present = std::collections::BTreeSet::new();
        for _ in 0..count {
            let op = next() as u32;
            let mut result = 0.0;
            match op {
                0 => {
                    let id = next() as u64;
                    let idle = next();
                    let amount = next();
                    let maximum = next();
                    let priority = next() as i32;
                    let flow = next() as usize;
                    let public = next() != 0.0;
                    let saved = next();
                    let mut store = Store::finite(amount, maximum, priority);
                    store.flow = [Flow::None, Flow::Read, Flow::Write, Flow::ReadWrite][flow];
                    store.public = public;
                    system
                        .service_mut(current)
                        .unwrap()
                        .add_node(
                            id,
                            EnergyNode {
                                idle,
                                store: Some(store),
                                passive_rate: None,
                            },
                            saved,
                        )
                        .unwrap();
                    present.insert((current, id));
                }
                1 => {
                    let id = next() as u64;
                    result = system.remove_node(current, id).unwrap().saved_buffer;
                    present.remove(&(current, id));
                }
                2 => {
                    let amount = next();
                    let commit = next() != 0.0;
                    result = system.inject(current, amount, commit);
                }
                3 => {
                    let amount = next();
                    let commit = next() != 0.0;
                    let multiplier = next();
                    result = system.extract(current, amount, multiplier, commit);
                }
                4 => {
                    let id = next() as u64;
                    let value = next();
                    system
                        .service_mut(current)
                        .unwrap()
                        .set_storage_amount(id, value)
                        .unwrap();
                }
                5 => {
                    let id = next() as u64;
                    let value = next();
                    system
                        .service_mut(current)
                        .unwrap()
                        .set_idle(id, value)
                        .unwrap();
                }
                6 => {
                    let ticks = next() as usize;
                    let channel = next();
                    let multiplier = next();
                    for _ in 0..ticks {
                        system.begin_tick();
                        system.end_tick(current, channel, multiplier);
                    }
                }
                7 => system.service_mut(current).unwrap().refresh_power(),
                8 => result = system.demand(current, next()),
                9 => {
                    current = next() as u64;
                    if system.service(current).is_none() {
                        system
                            .add_service(current, EnergyService::default())
                            .unwrap();
                    }
                }
                10 => system.connect(next() as u64, next() as u64).unwrap(),
                11 => assert!(system.disconnect(next() as u64, next() as u64)),
                12 => {
                    let id = next() as u64;
                    let rate = next();
                    system
                        .service_mut(current)
                        .unwrap()
                        .add_node(
                            id,
                            EnergyNode {
                                idle: 0.0,
                                store: None,
                                passive_rate: Some(rate),
                            },
                            0.0,
                        )
                        .unwrap();
                    present.insert((current, id));
                }
                13 => {
                    system.end_tick(current, next(), next());
                }
                _ => panic!("unknown oracle operation"),
            }
            let service = system.service_mut(current).unwrap();
            print!(
                "{result:.17e} {:.17e} {:.17e} {:.17e} {:.17e} {:.17e} {:.17e} {}",
                service.buffer_amount(),
                service.node_energy_share(),
                service.maximum(),
                service.stored_power(),
                service.average_drain(),
                service.average_injection(),
                u8::from(service.is_powered())
            );
            for (owner, id) in &present {
                if *owner == current {
                    print!(" {:.17e}", service.store(*id).map_or(0.0, Store::amount));
                }
            }
            println!();
        }
    }
}
