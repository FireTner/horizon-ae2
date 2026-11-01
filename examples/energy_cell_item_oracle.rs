use ae2_horizon::ae2_item_energy::{ItemEnergy, energy_cell_model_level};
use std::io::{self, BufRead};
fn bits(v: f64) -> u64 {
    if v.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        v.to_bits()
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let line = line?;
        let row: Vec<u64> = line
            .split_ascii_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        if row.len() != 6 {
            return Err("expected kind present stored_bits extract amount_bits commit".into());
        }
        let capacity = if row[0] == 0 { 200000.0 } else { 1600000.0 };
        let mut item =
            ItemEnergy::from_energy_cell(capacity, (row[1] != 0).then(|| f64::from_bits(row[2])));
        let amount = f64::from_bits(row[4]);
        let result = if row[3] == 0 {
            item.inject(amount, row[5] != 0)
        } else {
            item.extract(amount, row[5] != 0)
        };
        println!(
            "{} {} {} {} {}",
            bits(result),
            bits(item.current()),
            u8::from(item.stored_component().is_some()),
            energy_cell_model_level(item.current(), capacity),
            if row[0] == 0 { 800 } else { 1600 }
        );
    }
    Ok(())
}
