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
    // TODO: doc
    pub fn is_24_7(&self) -> bool {
        let Some(kind) = self.rules.last().map(|rs| rs.kind) else {
            return true;
        };

        // TODO: are all kind of suffix OK ?
        // TODO: maybe base on normalize && ensure this is cached
        let Some(tail) = (self.rules.iter().rev()).find(|rs| {
            rs.day_selector.is_empty() || !rs.time_selector.is_00_24() || rs.kind != kind
        }) else {
            return kind == RuleKind::Closed;
        };

        tail.kind == kind && tail.is_24_7()
    }

    // TODO: doc
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

            let paving =
                (selector_seq.into_iter()).fold(Paving5D::default(), |mut union, selector| {
                    let full_day_selector = selector.unpack().1.clone().dim([FULL_TIME]);
                    union.set(&full_day_selector, false);
                    union.set(&selector, true);
                    union
                });

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
                RuleOperator::Normal => " ; ",
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
    /// TODO: more docs & examples
    ///
    /// If this returns `true`, then this expression is always open, but it
    /// can't detect all cases.
    pub fn is_24_7(&self) -> bool {
        self.day_selector.is_empty() && self.time_selector.is_00_24()
    }
}

impl Display for RuleSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_empty = true;

        if self.is_24_7() {
            is_empty = false;
            write!(f, "24/7")?;
        } else {
            is_empty = is_empty && self.day_selector.is_empty();
            write!(f, "{}", self.day_selector)?;

            if !self.time_selector.is_00_24() {
                if !is_empty {
                    write!(f, " ")?;
                }

                is_empty = is_empty && self.time_selector.is_00_24();
                write!(f, "{}", self.time_selector)?;
            }
        }

        if self.kind != RuleKind::Open {
            if !is_empty {
                write!(f, " ")?;
            }

            write!(f, "{}", self.kind)?;
        }

        Ok(())
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
