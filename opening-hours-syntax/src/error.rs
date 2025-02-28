use std::fmt;

use crate::parser::Rule;
use crate::rules::day::{WeekNum, WeekRange, Year, YearRange};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Error {
    Parser(Box<pest::error::Error<Rule>>),
    Unsupported(&'static str),
    Overflow {
        value: String,
        expected: String,
    },
    InvalidExtendTime {
        hour: u8,
        minutes: u8,
    },
    InvertedYearRange {
        start: Year,
        end: Year,
        step: u16,
    },
    InvertedWeekRange {
        start: WeekNum,
        end: WeekNum,
        step: u8,
    },
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
            Self::Parser(pest_err) => write!(f, "{pest_err}"),
            Self::Unsupported(desc) => write!(f, "using an unsupported feature: {desc}"),
            Self::InvalidExtendTime { hour, minutes: minute } => {
                write!(f, "invalid extended time for {hour:02}:{minute:02}")
            }
            Self::Overflow { value, expected } => {
                write!(f, "{value} is too large: expected {expected}")
            }
            &Self::InvertedYearRange { start, end, step } => {
                write!(
                    f,
                    "Inverted year ranges are ambiguous, do you mean '{}'?",
                    YearRange::new(end..=start, step).unwrap(),
                )
            }
            &Self::InvertedWeekRange { start, end, step } => {
                write!(
                    f,
                    "Inverted week ranges are ambiguous, do you mean '{}'?",
                    WeekRange::new(end..=start, step).unwrap(),
                )
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parser(err) => Some(err as _),
            _ => None,
        }
    }
}
