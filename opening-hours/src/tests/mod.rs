pub(crate) mod utils;

mod country;
mod eval_next_change;
mod eval_schedule;
mod eval_state;
mod localization;
mod performance;
mod regression_github;
mod regression_integration;

use std::str::FromStr;

use crate::OpeningHours;

fn sample() -> impl Iterator<Item = &'static str> {
    include_str!("data/sample.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

// TODO: rework
#[test]
fn parse_sample() {
    for raw_oh in sample() {
        eprintln!("Parse {raw_oh:?} from sample");
        assert!(OpeningHours::from_str(raw_oh).is_ok());
    }
}
