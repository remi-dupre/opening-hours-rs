pub mod day;
pub mod time;

use std::fmt::Display;
use std::sync::Arc;

use crate::rubik::Paving;
use crate::simplify::{canonical_to_seq, seq_to_canonical};
use crate::sorted_vec::UniqueSortedVec;

// OpeningHoursExpression

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OpeningHoursExpression {
    pub rules: Vec<RuleSequence>,
}

impl OpeningHoursExpression {
    pub fn simplify(self) -> Self {
        let mut rules_queue = self.rules.into_iter().peekable();
        let mut simplified = Vec::new();

        while let Some(head) = rules_queue.next() {
            if head.operator != RuleOperator::Normal && head.operator != RuleOperator::Additional {
                simplified.push(head);
                break;
            }

            let Some(mut canonical) = seq_to_canonical(&head) else {
                simplified.push(head);
                break;
            };

            while rules_queue.peek().is_some()
                && rules_queue.peek().unwrap().operator == head.operator
                && rules_queue.peek().unwrap().kind == head.kind
                && rules_queue.peek().unwrap().comments == head.comments
                && seq_to_canonical(rules_queue.peek().unwrap()).is_some()
            {
                canonical.union_with(
                    // TODO: computed twice
                    seq_to_canonical(&rules_queue.next().unwrap()).unwrap(),
                );
            }

            simplified.extend(canonical_to_seq(
                canonical,
                head.operator,
                head.kind,
                head.comments,
            ))
        }

        Self { rules: simplified }
    }
}

impl Display for OpeningHoursExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(first) = self.rules.first() else {
            return Ok(());
        };

        write!(f, "{first}")?;

        for rule in &self.rules[1..] {
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

// RuleSequence

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RuleSequence {
    pub day_selector: day::DaySelector,
    pub time_selector: time::TimeSelector,
    pub kind: RuleKind,
    pub operator: RuleOperator,
    pub comments: UniqueSortedVec<Arc<str>>,
}

impl RuleSequence {
    /// If this returns `true`, then this expression is always open, but it
    /// can't detect all cases.
    pub(crate) fn is_24_7(&self) -> bool {
        self.day_selector.is_empty()
            && self.time_selector.is_00_24()
            && self.operator == RuleOperator::Normal
    }
}

impl Display for RuleSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_24_7() {
            return write!(f, "24/7 {}", self.kind);
        }

        write!(f, "{}", self.day_selector)?;

        if !self.day_selector.is_empty() && !self.time_selector.time.is_empty() {
            write!(f, " ")?;
        }

        if !self.time_selector.is_00_24() {
            write!(f, "{} ", self.time_selector)?;
        }

        write!(f, "{}", self.kind)
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
