use ae2_horizon::ae2_controller;
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let line = line?;
        let row: Vec<i32> = line
            .split_ascii_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        if row.len() != 7 || !(0..64).contains(&row[3]) || !(0..3).contains(&row[6]) {
            return Err("expected x y z neighbour_mask powered conflict previous_state".into());
        }
        let neighbours = std::array::from_fn(|i| row[3] & (1 << i) != 0);
        println!(
            "{} {}",
            ae2_controller::render_type([row[0], row[1], row[2]], neighbours),
            ae2_controller::status(row[4] != 0, row[5] != 0)
        );
    }
    Ok(())
}
