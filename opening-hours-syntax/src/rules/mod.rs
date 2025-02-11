pub mod day;
pub mod time;

use std::fmt::Display;
use std::sync::Arc;

use crate::rubik::{Paving, Paving5D};
use crate::simplify::{canonical_to_seq, ruleseq_to_selector, FULL_TIME};
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
            // TODO: implement addition and fallback
            if head.operator != RuleOperator::Normal {
                simplified.push(head);
                continue;
            }

            let Some(selector) = ruleseq_to_selector(&head) else {
                simplified.push(head);
                continue;
            };

            let mut selector_seq = vec![selector];

            while let Some(selector) = rules_queue
                .peek()
                .filter(|r| r.operator == head.operator)
                .filter(|r| r.kind == head.kind)
                .filter(|r| r.comments == head.comments)
                .and_then(ruleseq_to_selector)
            {
                rules_queue.next();
                selector_seq.push(selector);
            }

            let paving = (selector_seq.into_iter().rev()).fold(
                Paving5D::default(),
                |mut union, selector| {
                    let full_day_selector = selector.unpack().1.clone().dim([FULL_TIME]);
                    union.set(&full_day_selector, false);
                    union.set(&selector, true);
                    union
                },
            );

            simplified.extend(canonical_to_seq(
                paving,
                head.operator,
                head.kind,
                head.comments,
            ));
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
