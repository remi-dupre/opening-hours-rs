#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

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
