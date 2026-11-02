use ae2_horizon::ae2_part_use::{self, Context, UseResult};
use std::io::{self, BufRead};
struct Case {
    part: bool,
    client: bool,
    player: bool,
    creative: bool,
    count: i32,
    target: bool,
    added: bool,
    empty: bool,
    events: String,
}
impl Context for Case {
    type Placement = ();
    fn is_part(&self) -> bool {
        self.part
    }
    fn client_side(&self) -> bool {
        self.client
    }
    fn placement(&mut self) -> Option<()> {
        self.events.push('T');
        self.target.then_some(())
    }
    fn place(&mut self, _: ()) -> bool {
        self.events.push('P');
        self.added
    }
    fn immediate_update(&mut self) {
        self.events.push('U');
    }
    fn consume(&mut self) {
        if self.player && !self.creative {
            self.events.push('C');
            self.count -= 1;
            if self.count == 0 {
                self.empty = true;
                self.events.push('E');
            }
        }
    }
}
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let a: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let mut c = Case {
            part: a[0] != 0,
            client: a[1] != 0,
            player: a[2] != 0,
            creative: a[3] != 0,
            count: a[4],
            target: a[5] != 0,
            added: a[6] != 0,
            empty: false,
            events: String::new(),
        };
        let result = ae2_part_use::use_item(&mut c);
        let name = match result {
            UseResult::Pass => "PASS",
            UseResult::Fail => "FAIL",
            UseResult::Success => "SUCCESS",
            UseResult::Consume => "CONSUME",
        };
        println!(
            "{name} {} {} {} {} {}",
            c.count,
            u8::from(c.empty),
            if c.events.is_empty() { "-" } else { &c.events },
            u8::from(result.stops_click()),
            u8::from(result.swings())
        );
    }
}
