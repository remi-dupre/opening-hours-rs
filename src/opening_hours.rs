use std::cmp::{max, min};
use std::convert::TryInto;
use std::fmt::Display;
use std::iter::Peekable;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::{OpeningHoursExpression, RuleKind, RuleOperator, RuleSequence};

use crate::context::Context;
use crate::date_filter::DateFilter;
use crate::error::ParserError;
use crate::schedule::{FilledScheduleIterator, Schedule};
use crate::time_filter::{time_selector_intervals_at, time_selector_intervals_at_next_day};
use crate::DateTimeRange;

/// The upper bound of dates handled by specification
pub const DATE_LIMIT: NaiveDateTime = {
    let Some(date) = NaiveDate::from_ymd_opt(10_000, 1, 1) else {
        panic!("Invalid limit date")
    };

    let Some(time) = NaiveTime::from_hms_opt(0, 0, 0) else {
        panic!("Invalid limit time")
    };

    NaiveDateTime::new(date, time)
};

// OpeningHours

/// A parsed opening hours expression and its evaluation context.
///
/// Note that all big inner structures are immutable and wrapped by an `Arc`
/// so this is safe and fast to clone.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OpeningHours {
    /// Rules describing opening hours
    expr: Arc<OpeningHoursExpression>,
    /// Evalutation context
    ctx: Context,
}

impl OpeningHours {
    // --
    // -- Builder Methods
    // --

    /// Parse a raw opening hours expression.
    ///
    /// # Example
    ///
    /// ```
    /// use opening_hours::OpeningHours;
    ///
    /// assert!(OpeningHours::parse("24/7 open").is_ok());
    /// assert!(OpeningHours::parse("not a valid expression").is_err());
    /// ```
    pub fn parse(raw_oh: &str) -> Result<Self, ParserError> {
        let expr = Arc::new(opening_hours_syntax::parse(raw_oh)?);
        Ok(Self { expr, ctx: Context::default() })
    }

    /// Set a new evaluation context for this expression.
    ///
    /// # Example
    ///
    /// ```
    /// use opening_hours::OpeningHours;
    /// use opening_hours::context::Context;
    ///
    /// let oh = OpeningHours::parse("Mo-Fr open")
    ///     .unwrap()
    ///     .with_context(Context::default());
    /// ```
    pub fn with_context(mut self, ctx: Context) -> Self {
        self.ctx = ctx;
        self
    }

    // --
    // -- Low level implementations.
    // --
    //
    // Following functions are used to build the TimeDomainIterator which is
    // used to implement all other functions.
    //
    // This means that performances matters a lot for these functions and it
    // would be relevant to focus on optimisations to this regard.

    /// Provide a lower bound to the next date when a different set of rules
    /// could match.
    fn next_change_hint(&self, date: NaiveDate) -> Option<NaiveDate> {
        (self.expr.rules)
            .iter()
            .map(|rule| rule.day_selector.next_change_hint(date, &self.ctx))
            .min()
            .flatten()
    }

    /// TODO: doc
    pub fn schedule_at(&self, date: NaiveDate) -> Schedule {
        let mut prev_match = false;
        let mut prev_eval = None;

        for rules_seq in &self.expr.rules {
            let curr_match = rules_seq.day_selector.filter(date, &self.ctx);
            let curr_eval = rule_sequence_schedule_at(rules_seq, date, &self.ctx);

            let (new_match, new_eval) = match rules_seq.operator {
                RuleOperator::Normal => (
                    curr_match || prev_match,
                    if curr_match {
                        curr_eval
                    } else {
                        prev_eval.or(curr_eval)
                    },
                ),
                RuleOperator::Additional => (
                    prev_match || curr_match,
                    match (prev_eval, curr_eval) {
                        (Some(prev), Some(curr)) => Some(prev.addition(curr)),
                        (prev, curr) => prev.or(curr),
                    },
                ),
                RuleOperator::Fallback => {
                    if prev_match {
                        (prev_match, prev_eval)
                    } else {
                        (curr_match, curr_eval)
                    }
                }
            };

            prev_match = new_match;
            prev_eval = new_eval;
        }

        prev_eval.unwrap_or_else(Schedule::empty)
    }

    /// TODO: doc
    pub fn iter_range(
        &self,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> impl Iterator<Item = DateTimeRange> + Send + Sync {
        let from = std::cmp::min(DATE_LIMIT, from);
        let to = std::cmp::min(DATE_LIMIT, to);

        TimeDomainIterator::new(self, from, to)
            .take_while(move |dtr| dtr.range.start < to)
            .map(move |dtr| {
                let start = max(dtr.range.start, from);
                let end = min(dtr.range.end, to);
                DateTimeRange::new_with_sorted_comments(start..end, dtr.kind, dtr.comments)
            })
    }

    // TODO: doc
    pub fn iter_from(
        &self,
        from: NaiveDateTime,
    ) -> impl Iterator<Item = DateTimeRange> + Send + Sync {
        self.iter_range(from, DATE_LIMIT)
    }

    // --
    // -- High level implementations
    // --

    /// TODO: doc
    pub fn next_change(&self, current_time: NaiveDateTime) -> Option<NaiveDateTime> {
        let interval = self.iter_from(current_time).next()?;

        if interval.range.end == DATE_LIMIT {
            None
        } else {
            Some(interval.range.end)
        }
    }

    /// TODO: doc
    pub fn state(&self, current_time: NaiveDateTime) -> RuleKind {
        self.iter_range(current_time, current_time + Duration::minutes(1))
            .next()
            .map(|dtr| dtr.kind)
            .unwrap_or(RuleKind::Unknown)
    }

    /// TODO: doc
    pub fn is_open(&self, current_time: NaiveDateTime) -> bool {
        self.state(current_time) == RuleKind::Open
    }

    /// TODO: doc
    pub fn is_closed(&self, current_time: NaiveDateTime) -> bool {
        self.state(current_time) == RuleKind::Closed
    }

    /// TODO: doc
    pub fn is_unknown(&self, current_time: NaiveDateTime) -> bool {
        self.state(current_time) == RuleKind::Unknown
    }
}

impl FromStr for OpeningHours {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Display for OpeningHours {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)
    }
}

fn rule_sequence_schedule_at(
    rule_sequence: &RuleSequence,
    date: NaiveDate,
    ctx: &Context,
) -> Option<Schedule> {
    let from_today = Some(date)
        .filter(|date| rule_sequence.day_selector.filter(*date, ctx))
        .map(|date| time_selector_intervals_at(&rule_sequence.time_selector, date))
        .map(|rgs| Schedule::from_ranges(rgs, rule_sequence.kind, &rule_sequence.comments));

    let from_yesterday = (date.pred_opt())
        .filter(|prev| rule_sequence.day_selector.filter(*prev, ctx))
        .map(|prev| time_selector_intervals_at_next_day(&rule_sequence.time_selector, prev))
        .map(|rgs| Schedule::from_ranges(rgs, rule_sequence.kind, &rule_sequence.comments));

    match (from_today, from_yesterday) {
        (Some(sched_1), Some(sched_2)) => Some(sched_1.addition(sched_2)),
        (today, yesterday) => today.or(yesterday),
    }
}

// TimeDomainIterator

pub struct TimeDomainIterator {
    opening_hours: OpeningHours,
    curr_date: NaiveDate,
    curr_schedule: Peekable<FilledScheduleIterator>,
    end_datetime: NaiveDateTime,
}

impl TimeDomainIterator {
    /// TODO: doc
    pub fn new(
        opening_hours: &OpeningHours,
        start_datetime: NaiveDateTime,
        end_datetime: NaiveDateTime,
    ) -> Self {
        let opening_hours = opening_hours.clone();
        let start_date = start_datetime.date();
        let start_time = start_datetime.time().into();

        let mut curr_schedule = opening_hours
            .schedule_at(start_date)
            .into_iter_filled()
            .peekable();

        if start_datetime >= end_datetime {
            (&mut curr_schedule).for_each(|_| {});
        }

        while curr_schedule
            .peek()
            .map(|tr| !tr.range.contains(&start_time))
            .unwrap_or(false)
        {
            curr_schedule.next();
        }

        Self {
            opening_hours,
            curr_date: start_date,
            curr_schedule,
            end_datetime,
        }
    }

    fn consume_until_next_kind(&mut self, curr_kind: RuleKind) {
        while self.curr_schedule.peek().map(|tr| tr.kind) == Some(curr_kind) {
            self.curr_schedule.next();

            if self.curr_schedule.peek().is_none() {
                let next_change_hint = self
                    .opening_hours
                    .next_change_hint(self.curr_date)
                    .unwrap_or_else(|| self.curr_date.succ_opt().expect("reached invalid date"));

                assert!(next_change_hint > self.curr_date);
                self.curr_date = next_change_hint;

                if self.curr_date < self.end_datetime.date() {
                    self.curr_schedule = self
                        .opening_hours
                        .schedule_at(self.curr_date)
                        .into_iter_filled()
                        .peekable();
                }
            }
        }
    }
}

impl Iterator for TimeDomainIterator {
    type Item = DateTimeRange;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(curr_tr) = self.curr_schedule.peek().cloned() {
            let start = NaiveDateTime::new(
                self.curr_date,
                curr_tr
                    .range
                    .start
                    .try_into()
                    .expect("got invalid time from schedule"),
            );

            self.consume_until_next_kind(curr_tr.kind);

            let end_date = self.curr_date;
            let end_time = self
                .curr_schedule
                .peek()
                .map(|tr| tr.range.start)
                .unwrap_or_else(|| ExtendedTime::new(0, 0));

            let end = std::cmp::min(
                self.end_datetime,
                NaiveDateTime::new(
                    end_date,
                    end_time.try_into().expect("got invalid time from schedule"),
                ),
            );

            Some(DateTimeRange::new_with_sorted_comments(
                start..end,
                curr_tr.kind,
                curr_tr.comments,
            ))
        } else {
            None
        }
    }
}
