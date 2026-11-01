// SPDX-License-Identifier: LGPL-3.0-or-later
use ae2_horizon::{ae2_charger, ae2_energy::MIN_AMOUNT, ae2_item_energy::ItemEnergy};
use std::fmt::Write;
fn main() {
    use std::io::Read;
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text).unwrap();
    let mut words = text.split_whitespace();
    let mut next = || words.next().unwrap();
    let cases: usize = next().parse().unwrap();
    let mut output = String::new();
    for _ in 0..cases {
        let maximum = next().parse().unwrap();
        let current = next().parse().unwrap();
        let override_value: f64 = next().parse().unwrap();
        let mut local: f64 = next().parse().unwrap();
        let mut grid: f64 = next().parse().unwrap();
        let rate: f64 = next().parse().unwrap();
        let factor: f64 = next().parse().unwrap();
        let multiplier: f64 = next().parse().unwrap();
        let count: usize = next().parse().unwrap();
        let mut item = ItemEnergy::from_components(
            maximum,
            (override_value >= 0.0).then_some(override_value),
            Some(current),
        );
        for _ in 0..count {
            let ticks = next().parse().unwrap();
            let mut requests = Vec::new();
            let mut extract = |amount: f64| {
                if grid < 0.0 {
                    return 0.0;
                }
                requests.push(amount);
                if amount < MIN_AMOUNT {
                    return 0.0;
                }
                let result = amount.min(grid);
                grid -= result;
                if grid < MIN_AMOUNT {
                    grid = 0.0;
                }
                result
            };
            let working = item.maximum() > 0.0
                && ae2_charger::charge_item(
                    &mut local,
                    &mut item,
                    rate,
                    ticks,
                    factor,
                    multiplier,
                    &mut extract,
                );
            if local < ae2_charger::POWER_THRESHOLD {
                let extracted = extract(800.0f64.min(1600.0 - local));
                if extracted >= MIN_AMOUNT {
                    local += extracted.min(1600.0 - local);
                    if local < MIN_AMOUNT {
                        local = 0.0;
                    }
                }
            }
            writeln!(
                output,
                "{} {} {} {} {} {} {}",
                item.current(),
                local,
                grid,
                u8::from(working),
                requests.len(),
                requests.first().unwrap_or(&0.0),
                requests.get(1).unwrap_or(&0.0)
            )
            .unwrap();
        }
    }
    print!("{output}");
}
