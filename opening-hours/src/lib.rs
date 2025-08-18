#![doc = include_str!("../../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[macro_use]
pub mod schedule;
pub mod localization;

mod context;
mod filter;
mod opening_hours;
mod utils;

#[cfg(test)]
mod tests;

// Public re-exports
// TODO: make opening_hours.rs lighter and less spaghetty
pub use crate::context::{Context, ContextHolidays};
pub use crate::opening_hours::{OpeningHours, DATE_END};
pub use crate::utils::range::DateTimeRange;
pub use opening_hours_syntax::rules::RuleKind;
