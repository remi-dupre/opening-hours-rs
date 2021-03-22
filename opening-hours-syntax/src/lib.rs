//! Parsing component of [opening-hours] crate.
//!
//!
//! Usage
//! -----
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! opening-hours-syntax = "0"
//! ```
//!
//! And then a basic usage would look like that:
//!
//! ```rust
//! use opening_hours_syntax::parse;
//!
//! // Opens until 18pm during the week and until 12am the week-end.
//! const OH: &str = "Mo-Fr 10:00-18:00; Sa-Su 10:00-12:00";
//!
//! fn main() {
//!     let oh = parse(&OH).unwrap();
//!     eprintln!("{:?}", oh);
//! }
//! ```
//!
//!
//!
//! [opening-hours]: https://crates.io/crates/opening-hours
//!     "Root Package"

#[macro_use]
extern crate pest_derive;

pub mod error;
pub mod extended_time;
pub mod rules;
pub mod sorted_vec;

mod parser;

pub use error::{Error, Result};
pub use parser::parse;
