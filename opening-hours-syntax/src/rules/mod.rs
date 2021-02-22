pub mod day;
pub mod time;

// RuleSequence

#[derive(Clone, Debug)]
pub struct RuleSequence {
    pub day_selector: day::DaySelector,
    pub time_selector: time::TimeSelector,
    pub kind: RuleKind,
    pub operator: RuleOperator,
    comments: Vec<String>,
}

impl RuleSequence {
    pub fn new(
        day_selector: day::DaySelector,
        time_selector: time::TimeSelector,
        kind: RuleKind,
        operator: RuleOperator,
        mut comments: Vec<String>,
    ) -> Self {
        comments.sort_unstable();

        Self {
            day_selector,
            time_selector,
            kind,
            operator,
            comments,
        }
    }

    /// Return the sorted list of comments attached to this RuleSequence.
    pub fn comments(&self) -> &[String] {
        self.comments.as_slice()
    }
}

// RuleKind

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuleKind {
    Open,
    Closed,
    Unknown,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RuleOperator {
    Normal,
    Additional,
    Fallback,
}
