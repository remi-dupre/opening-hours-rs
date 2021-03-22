pub mod day;
pub mod time;

use crate::sorted_vec::UniqueSortedVec;

// RuleSequence

#[derive(Clone, Debug)]
pub struct RuleSequence {
    pub day_selector: day::DaySelector,
    pub time_selector: time::TimeSelector,
    pub kind: RuleKind,
    pub operator: RuleOperator,
    pub comments: UniqueSortedVec<String>,
}

// RuleKind

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuleKind {
    Open,
    Closed,
    Unknown,
}

// RuleOperator

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RuleOperator {
    Normal,
    Additional,
    Fallback,
}
