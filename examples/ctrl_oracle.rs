use ae2_horizon::ae2_ctrl::{Controls, decode, encode};
use std::io::{self, BufRead};
fn main() {
    let mut controls = Controls::default();
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut values = line.split_whitespace();
        let sender: u128 = values.next().unwrap().parse().unwrap();
        let data: Vec<u8> = values.map(|s| s.parse().unwrap()).collect();
        let result = decode(&data);
        controls.receive(sender, &data);
        match result {
            Some(down) => print!("{}", encode(down)[0]),
            None => print!("bad"),
        }
        for id in 0..4 {
            print!(" {}", u8::from(controls.down(id)));
        }
        println!();
    }
}
