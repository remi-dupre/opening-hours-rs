#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;
extern crate pest_derive;

mod display;
mod normalize;
mod parser;
mod util;

pub mod error;
pub mod extended_time;
pub mod rules;
pub mod warning;

pub use error::{Error, Result};
pub use extended_time::ExtendedTime;
pub use parser::{parse, Parser};
pub use rules::RuleKind;
pub use warning::Warning;

#[cfg(test)]
pub mod tests;
