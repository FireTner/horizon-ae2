use ae2_horizon::ae2_energy::{cell_signal, display_level};
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let line = line?;
        let row: Vec<u64> = line
            .split_ascii_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        if row.len() != 2 {
            return Err("expected current/max double bit patterns".into());
        }
        let current = f64::from_bits(row[0]);
        let maximum = f64::from_bits(row[1]);
        println!(
            "{} {}",
            display_level(current / maximum),
            cell_signal(current, maximum)
        );
    }
    Ok(())
}
