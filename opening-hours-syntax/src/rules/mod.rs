pub mod day;
pub mod time;

use std::fmt::Display;
use std::sync::Arc;

use crate::normalize::{Bounded, canonical_to_seq, ruleseq_to_selector};
use crate::rubik::{Paving, Paving5D};
use crate::sorted_vec::UniqueSortedVec;

// OpeningHoursExpression

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OpeningHoursExpression {
    pub rules: Vec<RuleSequence>,
}

impl OpeningHoursExpression {
    /// Check if this expression is *trivially* constant (ie. always evaluated at the exact same
    /// status). Note that this may return `false` for an expression that is constant but should
    /// cover most common cases.
    ///
    /// ```
    /// use opening_hours_syntax::parse;
    ///
    /// assert!(parse("24/7").unwrap().is_constant());
    /// assert!(parse("24/7 closed").unwrap().is_constant());
    /// assert!(parse("00:00-24:00 open").unwrap().is_constant());
    /// assert!(!parse("00:00-18:00 open").unwrap().is_constant());
    /// assert!(!parse("24/7 ; PH off").unwrap().is_constant());
    /// ```
    pub fn is_constant(&self) -> bool {
        let Some(kind) = self.rules.last().map(|rs| rs.kind) else {
            return true;
        };

        // Ignores rules from the end as long as they are all evaluated to the same kind.
        let search_tail_full = self.rules.iter().rev().find(|rs| {
            rs.day_selector.is_empty() || !rs.time_selector.is_00_24() || rs.kind != kind
        });

        let Some(tail) = search_tail_full else {
            return false;
        };

        tail.kind == kind && tail.is_constant()
    }

    // TODO: doc
    pub fn normalize(self) -> Self {
        let mut rules_queue = self.rules.into_iter().peekable();
        let mut normalized = Vec::new();

        while let Some(head) = rules_queue.next() {
            // TODO: implement addition and fallback
            if head.operator != RuleOperator::Normal {
                normalized.push(head);
                continue;
            }

            let Some(selector) = ruleseq_to_selector(&head) else {
                normalized.push(head);
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
                    let full_day_selector = selector.unpack().1.clone().dim([Bounded::bounds()]);
                    union.set(&full_day_selector, false);
                    union.set(&selector, true);
                    union
                });

            normalized.extend(canonical_to_seq(
                paving,
                head.operator,
                head.kind,
                head.comments,
            ));
        }

        Self { rules: normalized }
    }
}

impl Display for OpeningHoursExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(first) = self.rules.first() else {
            return write!(f, "closed");
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
    pub fn is_constant(&self) -> bool {
        self.day_selector.is_empty() && self.time_selector.is_00_24()
    }
}

impl Display for RuleSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_empty = true;

        if self.is_constant() {
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
