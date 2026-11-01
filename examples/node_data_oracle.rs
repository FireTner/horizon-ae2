use ae2_horizon::ae2_node_data::same_ignoring_owner;
use std::collections::BTreeMap;
use std::io::{self, BufRead, BufWriter, Write};
fn map(tokens: &mut std::str::SplitWhitespace<'_>) -> Option<BTreeMap<String, String>> {
    let size: i32 = tokens.next().unwrap().parse().unwrap();
    if size < 0 {
        return None;
    }
    Some(
        (0..size)
            .map(|_| (tokens.next().unwrap().into(), tokens.next().unwrap().into()))
            .collect(),
    )
}
fn main() {
    let mut out = BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut tokens = line.split_whitespace();
        let a = map(&mut tokens);
        let b = map(&mut tokens);
        writeln!(
            out,
            "{}",
            same_ignoring_owner(a.as_ref(), b.as_ref(), &"p".to_owned()) as u8
        )
        .unwrap();
        assert!(tokens.next().is_none());
    }
}
