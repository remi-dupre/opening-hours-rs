use chrono::NaiveDateTime;

use crate::opening_hours::DATE_LIMIT;

// TODO: doc
#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Error {
    // TODO: doc
    #[error("parser error: {0}")]
    Parser(#[from] opening_hours_syntax::error::Error),
    // TODO: doc
    #[error("date limit exceeded: {0} ≥ {}", *DATE_LIMIT)]
    DateLimitExceeded(NaiveDateTime),
    // TODO: doc
    #[error("could find region `{0}`")]
    RegionNotFound(String),
}

// TODO: doc
pub type Result<T> = std::result::Result<T, Error>;
