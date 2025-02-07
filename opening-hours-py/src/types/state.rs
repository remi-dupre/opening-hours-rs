use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyclass_enum;

use opening_hours_syntax::rules::RuleKind;

/// Specify the state of an opening hours interval.
#[allow(clippy::upper_case_acronyms)]
#[gen_stub_pyclass_enum]
#[pyclass(ord, eq, frozen, hash, str)]
#[derive(Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum State {
    /// Currently open
    OPEN,
    /// Currently closed
    CLOSED,
    /// May be open depending on context
    UNKNOWN,
}

impl From<RuleKind> for State {
    fn from(kind: RuleKind) -> Self {
        match kind {
            RuleKind::Open => Self::OPEN,
            RuleKind::Closed => Self::CLOSED,
            RuleKind::Unknown => Self::UNKNOWN,
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::OPEN => write!(f, "open"),
            State::CLOSED => write!(f, "closed"),
            State::UNKNOWN => write!(f, "unknown"),
        }
    }
}
