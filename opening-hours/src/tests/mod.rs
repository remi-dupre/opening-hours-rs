use crate::OpeningHours;

pub(crate) mod utils;

mod country;
mod eval_next_change;
mod eval_schedule;
mod eval_state;
mod localization;
mod regression_github;
mod regression_integration;

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
        assert!(OpeningHours::parse(raw_oh).is_ok());
    }
}
