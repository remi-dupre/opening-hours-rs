extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate chrono;

pub mod day_selector;
pub mod parser;
#[macro_use]
pub mod schedule;
pub mod opening_hours;
pub mod time_selector;

mod utils;

#[cfg(test)]
mod tests;

// Public re-exports
// TODO: make opening_hours.rs lighter and less spaghetty
pub use opening_hours::OpeningHours;
pub use parser::parse;
