pub mod day;
pub mod time;

use std::fmt::Display;

use crate::sorted_vec::UniqueSortedVec;

// RuleSequence

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RuleSequence {
    pub day_selector: day::DaySelector,
    pub time_selector: time::TimeSelector,
    pub kind: RuleKind,
    pub operator: RuleOperator,
    pub comments: UniqueSortedVec<String>,
}

impl RuleSequence {
    /// Pretty print the rules set through given writer.
    pub fn write_rules_seq(
        f: &mut std::fmt::Formatter<'_>,
        seq: &[RuleSequence],
    ) -> std::fmt::Result {
        let Some(first) = seq.first() else {
            return Ok(());
        };

        write!(f, "{first}")?;

        for rule in &seq[1..] {
            let separator = match rule.operator {
                RuleOperator::Normal => "; ",
                RuleOperator::Additional => ", ",
                RuleOperator::Fallback => " || ",
            };

            write!(f, "{separator}{rule}")?;
        }

        Ok(())
    }
}

impl Display for RuleSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.day_selector)?;

        if !self.day_selector.is_empty() && !self.time_selector.time.is_empty() {
            write!(f, " ")?;
        }

        write!(f, "{} {}", self.time_selector, self.kind)
    }
}

// RuleKind

#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuleKind {
    Open,
    Closed,
    Unknown,
}

impl RuleKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Unknown => "unknown",
        }
    }
}

impl Display for RuleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// RuleOperator

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum RuleOperator {
    Normal,
    Additional,
    Fallback,
}
