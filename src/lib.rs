#![doc = include_str!("../README.md")]

pub mod date_filter;
#[macro_use]
pub mod schedule;
pub mod opening_hours;
pub mod time_filter;

mod utils;

#[cfg(test)]
mod tests;

// Public re-exports
// TODO: make opening_hours.rs lighter and less spaghetty
pub use crate::opening_hours::OpeningHours;
pub use crate::utils::range::DateTimeRange;
pub use opening_hours_syntax::error::Error as ParserError;
pub use opening_hours_syntax::rules::RuleKind;
