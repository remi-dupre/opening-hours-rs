use std::fmt;

use crate::parser::Rule;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Error {
    Parser(Box<pest::error::Error<Rule>>),
    Unsupported(&'static str),
    Overflow { value: String, expected: String },
}

impl From<pest::error::Error<Rule>> for Error {
    #[inline]
    fn from(pest_err: pest::error::Error<Rule>) -> Self {
        Self::Parser(pest_err.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parser(pest_err) => write!(f, "{}", pest_err),
            Self::Unsupported(desc) => write!(f, "using an unsupported feature: {}", desc),
            Self::Overflow { value, expected } => {
                write!(f, "{} is too large: expected {}", value, expected)
            }
        }
    }
}
