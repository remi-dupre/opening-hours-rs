use alloc::boxed::Box;
use core::fmt;
use core::ops::RangeInclusive;

use crate::ExtendedTime;
use crate::parser::Rule;
use crate::rules::day::{WeekNum, Year};

pub type Result<T> = core::result::Result<T, Error>;

const REPORT_ISSUE_LINK: &str = "https://github.com/remi-dupre/opening-hours-rs/issues";

#[derive(Clone, Debug)]
pub enum Error {
    /// Could not parse the expression. It is not conformant with the grammar defined in the wiki:
    /// <https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification>
    Syntax(Box<pest::error::Error<Rule>>),
    /// Use of an unsupported feature.
    Unsupported(&'static str),
    /// Some number has overflowed.
    Overflow {
        value: i16,
        expected_bounds: RangeInclusive<i16>,
    },
    /// Extended time has overflowed.
    InvalidExtendedTime { hour: u8, minutes: u8 },
    /// A time range can't start from the next day
    TimeRangeStartsTomorrow(ExtendedTime),
    /// A year range that starts after it ends.
    InvertedYearRange { start: Year, end: Year, step: u16 },
    /// A week range that starts after it ends.
    InvertedWeekRange {
        start: WeekNum,
        end: WeekNum,
        step: u8,
    },
    /// This should never be built at runtime if the grammar implementation is sound.
    GrammarUnexpectedToken { rule: Rule, unexpected: Rule },
    /// This should never be built at runtime if the grammar implementation is sound.
    GrammarLogic { rule: Rule, invariant: &'static str },
}

impl Error {
    /// If this is true, this is an error that should not be raised as long as the implementation is
    /// sound. If this kind of error occurs, you can report it here :
    /// <https://github.com/remi-dupre/opening-hours-rs/issues>
    pub fn is_implementation_error(&self) -> bool {
        matches!(
            self,
            Self::GrammarUnexpectedToken { .. } | Self::GrammarLogic { .. }
        )
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(pest_err: pest::error::Error<Rule>) -> Self {
        Self::Syntax(pest_err.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(pest_err) => write!(f, "{pest_err}"),
            Self::Unsupported(desc) => write!(f, "using an unsupported feature: {desc}"),
            Self::InvalidExtendedTime { hour, minutes: minute } => {
                write!(f, "invalid extended time for {hour:02}:{minute:02}")
            }
            Self::TimeRangeStartsTomorrow(extended_time) => {
                write!(
                    f,
                    "a time range can't start from the next day (got {extended_time})"
                )
            }
            Self::Overflow { value, expected_bounds } => {
                write!(
                    f,
                    "{value} is too large: expected a value between {} and {}",
                    expected_bounds.start(),
                    expected_bounds.end(),
                )
            }
            &Self::InvertedYearRange { start, end, .. } => {
                write!(
                    f,
                    "Inverted year ranges are ambiguous, do you mean '{}-{}'?",
                    *end, *start
                )
            }
            &Self::InvertedWeekRange { start, end, .. } => {
                write!(
                    f,
                    "Inverted week ranges are ambiguous, do you mean '{}-{}'?",
                    *end, *start
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
            Self::Syntax(err) => Some(err as _),
            _ => None,
        }
    }
}

// --
// -- Helpers
// --

/// Commonly built errors
pub(crate) fn err_empty(rule: Rule) -> Error {
    Error::GrammarLogic { rule, invariant: "cannot be empty" }
}
