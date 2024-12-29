#![doc = include_str!("../../README.md")]
// Enable doc_auto_cfg feature when building docs on the nightly channel
// (which will be the case for docs.rs).
#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]

pub mod date_filter;
pub mod error;
#[macro_use]
pub mod schedule;
pub mod context;
pub mod country;
pub mod opening_hours;
pub mod time_filter;

mod utils;

#[cfg(test)]
mod tests;

// Public re-exports
// TODO: make opening_hours.rs lighter and less spaghetty
pub use crate::context::{Context, ContextHolidays, Localize, NoLocation, TzLocation};
pub use crate::opening_hours::OpeningHours;
pub use crate::utils::range::DateTimeRange;
pub use opening_hours_syntax::rules::RuleKind;
