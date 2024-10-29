#![doc = include_str!("../README.md")]

#[macro_use]
extern crate pest_derive;

pub mod error;
pub mod extended_time;
pub mod rules;
pub mod sorted_vec;

mod parser;

pub use error::{Error, Result};
pub use parser::parse;
