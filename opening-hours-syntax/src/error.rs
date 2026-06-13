use alloc::boxed::Box;
use core::fmt;
use core::ops::RangeInclusive;

use crate::parser::Rule;
use crate::rules::day::{WeekNum, WeekRange, Year, YearRange};

pub type Result<T> = core::result::Result<T, Error>;

const REPORT_ISSUE_LINK: &str = "https://github.com/remi-dupre/opening-hours-rs/issues";

#[derive(Clone, Debug)]
pub enum Error {
    Parser(Box<pest::error::Error<Rule>>),
    Unsupported(&'static str),
    Overflow {
        value: i16,
        expected_bounds: RangeInclusive<i16>,
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
    /// This should never be built at runtime if the grammar implementation is sound.
    GrammarUnexpectedToken {
        rule: Rule,
        unexpected: Rule,
    },
    /// This should never be built at runtime if the grammar implementation is sound.
    GrammarLogic {
        rule: Rule,
        invariant: &'static str,
    },
}

impl Error {
    /// If this is true, this is an error that should not be raised as long as the implementation is
    /// sound. If this kind of error occurs, you can report it here :
    /// https://github.com/remi-dupre/opening-hours-rs/issues
    pub fn is_implementation_error(&self) -> bool {
        matches!(
            self,
            Self::GrammarUnexpectedToken { .. } | Self::GrammarLogic { .. }
        )
    }
}

impl From<pest::error::Error<Rule>> for Error {
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
            Self::Overflow { value, expected_bounds } => {
                write!(
                    f,
                    "{value} is too large: expected a value between {} and {}",
                    expected_bounds.start(),
                    expected_bounds.end(),
                )
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
            Error::GrammarUnexpectedToken { rule, unexpected } => {
                write!(
                    f,
                    "Library implementation error in {rule:?}: found unexpected child {unexpected:?}. "
                )?;

                write!(f, "Please report an issue at {REPORT_ISSUE_LINK}.")
            }
            Self::GrammarLogic { rule, invariant: detail } => {
                write!(f, "Library implementation error in {rule:?}: {detail}. ")?;
                write!(f, "Please report an issue at {REPORT_ISSUE_LINK}.")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parser(err) => Some(err as _),
            _ => None,
        }
    }
}
