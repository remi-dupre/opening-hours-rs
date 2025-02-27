#![doc = include_str!("../README.md")]

#[macro_use]
extern crate pest_derive;

mod display;
mod normalize;
mod parser;

pub mod error;
pub mod extended_time;
pub mod rules;

pub use error::{Error, Result};
pub use extended_time::ExtendedTime;
pub use parser::parse;
pub use rules::RuleKind;

#[cfg(test)]
pub mod tests;
