// SPDX-License-Identifier: LGPL-3.0-or-later
use ae2_horizon::ae2_item_energy::ItemEnergy;
use std::fmt::Write;
fn main() {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let mut words = input.split_whitespace();
    let mut next = || words.next().unwrap();
    let cases: usize = next().parse().unwrap();
    let mut output = String::new();
    for _ in 0..cases {
        let default = next().parse().unwrap();
        let mut optional = || {
            let word = next();
            (word != "none").then(|| word.parse::<f64>().unwrap())
        };
        let mut energy = ItemEnergy::from_components(default, optional(), optional());
        let count: usize = next().parse().unwrap();
        for _ in 0..count {
            let result = match next() {
                op @ ("I" | "E") => {
                    let amount = next().parse().unwrap();
                    let commit = next() == "1";
                    if op == "I" {
                        energy.inject(amount, commit)
                    } else {
                        energy.extract(amount, commit)
                    }
                }
                "B" => energy.bar_width() as f64,
                "S" => {
                    energy.set_maximum(next().parse().unwrap());
                    0.0
                }
                "M" => {
                    energy.set_maximum_multiplier(next().parse().unwrap());
                    0.0
                }
                "R" => {
                    energy.reset_maximum();
                    0.0
                }
                "C" => {
                    energy.set_current(next().parse().unwrap());
                    0.0
                }
                op => panic!("unknown operation {op}"),
            };
            writeln!(
                output,
                "{} {} {} {} {}",
                result,
                energy.current(),
                energy.maximum(),
                u8::from(energy.stored_component().is_some()),
                u8::from(energy.capacity_override().is_some())
            )
            .unwrap();
        }
    }
    print!("{output}");
}
