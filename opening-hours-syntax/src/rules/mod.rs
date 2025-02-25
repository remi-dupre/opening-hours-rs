pub mod day;
pub mod time;

use std::fmt::Display;
use std::sync::Arc;

use crate::normalize::{
    canonical_to_seq, new_canonical_to_seq, ruleseq_to_day_selector, ruleseq_to_selector, Bounded,
    TimeSequenceList,
};
use crate::rubik::{DimFromBack, Paving, Paving4D, Paving5D};
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
            return kind == RuleKind::Closed;
        };

        tail.kind == kind && tail.is_constant()
    }

    /// Convert the expression into a normalized form. It will not affect the meaning of the
    /// expression and might impact the performance of evaluations.
    ///
    /// ```
    /// let oh = opening_hours_syntax::parse("24/7 ; Su closed").unwrap();
    /// assert_eq!(oh.normalize().to_string(), "Mo-Sa");
    /// ```
    pub fn old_normalize(self) -> Self {
        let mut rules_queue = self.rules.into_iter().peekable();
        let mut normalized = Vec::new();

        let Some(head) = rules_queue.next() else {
            return Self { rules: normalized };
        };

        // TODO: implement fallback
        if head.operator == RuleOperator::Fallback {
            normalized.push(head);
            normalized.extend(rules_queue);
            return Self { rules: normalized };
        }

        let Some(selector) = ruleseq_to_selector(&head) else {
            normalized.push(head);
            normalized.extend(rules_queue);
            return Self { rules: normalized };
        };

        let mut paving = Paving5D::default();
        paving.set(&selector, head.kind);

        while let Some(rule) = rules_queue.peek() {
            if rule.operator == RuleOperator::Fallback || rule.comments != head.comments {
                break;
            }

            let Some(selector) = ruleseq_to_selector(rule) else {
                break;
            };

            if rule.operator == RuleOperator::Normal && rule.kind != RuleKind::Closed {
                // If the rule is not explicitly targeting a closed kind, then it overrides
                // previous rules for the whole day.
                let (_, day_selector) = selector.clone().into_unpack_back();
                let full_day_selector = day_selector.dim_back([Bounded::bounds()]);
                paving.set(&full_day_selector, RuleKind::Closed);
            }

            paving.set(&selector, rule.kind);
            rules_queue.next();
        }

        normalized.extend(canonical_to_seq(paving, head.comments));
        normalized.extend(rules_queue);
        Self { rules: normalized }
    }

    pub fn normalize(self) -> Self {
        let mut rules_queue = self.rules.into_iter().peekable();
        let mut normalized = Vec::new();

        let Some(head) = rules_queue.next() else {
            return Self { rules: normalized };
        };

        // TODO: implement fallback & addition
        if head.operator == RuleOperator::Fallback || head.operator == RuleOperator::Additional {
            normalized.push(head);
            normalized.extend(rules_queue);
            return Self { rules: normalized };
        }

        let Some(selector) = ruleseq_to_day_selector(&head) else {
            normalized.push(head);
            normalized.extend(rules_queue);
            return Self { rules: normalized };
        };

        let mut paving = Paving4D::default();
        let (time_sequence, _) = TimeSequenceList::extract_from_rule_sequence(head);
        paving.set(&selector, time_sequence);

        while let Some(rule) = rules_queue.peek() {
            let Some(selector) = ruleseq_to_day_selector(rule) else {
                break;
            };

            let (time_sequence, _) = TimeSequenceList::extract_from_rule_sequence(rule.clone());

            match (rule.operator, rule.kind) {
                (RuleOperator::Additional, _) | (RuleOperator::Normal, RuleKind::Closed) => {
                    paving.add(&selector, &time_sequence)
                }
                (RuleOperator::Normal, RuleKind::Open | RuleKind::Unknown) => {
                    paving.set(&selector, time_sequence)
                }
                (RuleOperator::Fallback, _) => break,
            }

            rules_queue.next();
        }

        normalized.extend(new_canonical_to_seq(paving));
        normalized.extend(rules_queue);
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

        if !self.comments.is_empty() {
            write!(f, "\"{}\"", self.comments.join(", "))?;
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

impl Default for RuleKind {
    fn default() -> Self {
        Self::Closed
    }
}

impl Display for RuleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// RuleOperator

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum RuleOperator {
    Normal,
    Additional,
    Fallback,
}

impl Default for RuleOperator {
    fn default() -> Self {
        Self::Normal
    }
}
