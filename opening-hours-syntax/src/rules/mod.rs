pub mod day;
pub mod time;

use alloc::str::FromStr;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::Display;

use crate::normalize::{canonical_to_seq2, partialytocanonical2};

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
        let Some(state) = self.rules.last().map(|rs| rs.as_state()) else {
            return true;
        };

        // Ignores rules from the end as long as they are all evaluated to the same kind.
        let search_tail_full = self.rules.iter().rev().find(|rs| {
            rs.day_selector.is_empty() || !rs.time_selector.is_00_24() || rs.as_state() != state
        });

        let Some(tail) = search_tail_full else {
            return state == Default::default();
        };

        tail.as_state() == state && tail.is_constant()
    }

    /// Convert the expression into a normalized form. It will not affect the meaning of the
    /// expression and might impact the performance of evaluations.
    ///
    /// ```
    /// let oh = opening_hours_syntax::parse("24/7 ; Su closed").unwrap();
    /// assert_eq!(oh.normalize().to_string(), "Mo-Sa");
    /// ```
    pub fn normalize(self) -> Self {
        self.normalize_v2()
    }

    /// TODO: doc
    pub fn normalize_v2(self) -> Self {
        let mut old_rules = self.rules.into();
        let canonical = partialytocanonical2(&mut old_rules);
        let mut new_rules = canonical_to_seq2(canonical);
        new_rules.extend(old_rules);
        Self { rules: new_rules }
    }
}

impl Display for OpeningHoursExpression {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Some(first) = self.rules.first() else {
            return write!(f, "closed");
        };

        write!(f, "{first}")?;

        for rule in &self.rules[1..] {
            let separator = match rule.operator {
                RuleOperator::Normal => "; ",
                RuleOperator::Additional => ", ",
                RuleOperator::Fallback => " || ",
            };

            write!(f, "{separator}")?;

            // If the rule operatior is an addition, we need to make sure that the time selector is
            // prefixed with a day selector to avoid ambiguous syntax. For eg. "Mo 10:00-12:00,
            // 13:00-14:00" is parsed as a single rule while "Mo 10:00-12:00, Mo-Su 13:00-14:00"
            // isn't.
            rule.display(f, rule.operator == RuleOperator::Additional)?;
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
    pub comment: Arc<str>,
}

impl RuleSequence {
    /// If this returns `true`, then this expression is always open, but it
    /// can't detect all cases.
    pub fn is_constant(&self) -> bool {
        self.day_selector.is_empty() && self.time_selector.is_00_24()
    }

    /// Extract the kind and comment from the range, which are the values that define current state
    /// of an expression.
    pub fn as_state(&self) -> (RuleKind, &str) {
        (self.kind, &self.comment)
    }

    /// Format rule sequence into given formatter.
    ///
    /// If `force_day_selector` is set to true, the day selector part is guaranteed to yield a
    /// non-empty string by adding "Mo-Su" as fallback.
    pub(crate) fn display(
        &self,
        f: &mut core::fmt::Formatter<'_>,
        force_day_selector: bool,
    ) -> core::fmt::Result {
        let mut is_empty;

        if self.is_constant() {
            is_empty = false;
            write!(f, "24/7")?;
        } else {
            self.day_selector.display(f, force_day_selector)?;
            is_empty = !force_day_selector && self.day_selector.is_empty();

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

            is_empty = false;
            write!(f, "{}", self.kind)?;
        }

        if !self.comment.is_empty() {
            if !is_empty {
                write!(f, " ")?;
            }

            write!(f, "\"{}\"", self.comment)?;
        }

        Ok(())
    }
}

impl Display for RuleSequence {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.display(f, false)
    }
}

// RuleKind

#[derive(Copy, Clone, Debug, Default, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuleKind {
    Open,
    #[default]
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

impl FromStr for RuleKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            "unknown" => Ok(Self::Unknown),
            other => Err(format!("Unknown rule kind {other:?}")),
        }
    }
}

impl Display for RuleKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
