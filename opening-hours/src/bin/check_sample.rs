use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use opening_hours::OpeningHours;

fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("opening-hours")
        .join("src")
        .join("tests")
        .join("data")
        .join("sample.txt");

    let file = File::open(&path).expect("could not open file");
    let data = BufReader::new(file);
    let date = chrono::Utc::now();

    for expr in data.lines() {
        let expr = expr.expect("error while reading file");
        let oh: OpeningHours = expr.parse().expect("invalid expression");
        println!("- expr: {oh}");
        println!("  normalized: {}", oh.normalize());
        let (state, comment) = oh.state(date.naive_local());
        println!("  state: {state}");

        if !comment.is_empty() {
            println!("  {comment}");
        }

        if let Some(next_change) = oh.next_change(date.naive_local()) {
            println!("  next_change: {}", next_change);
        }
    }
}
