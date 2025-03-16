use std::fmt::Display;
use std::iter::Peekable;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::{OpeningHoursExpression, RuleKind, RuleOperator, RuleSequence};
use opening_hours_syntax::Error as ParserError;

use crate::filter::date_filter::{dates_with_potential_change, DateFilter};
use crate::filter::time_filter::{
    time_selector_intervals_at, time_selector_intervals_at_next_day, TimeFilter,
};
use crate::localization::{Localize, NoLocation};
use crate::schedule::Schedule;
use crate::Context;
use crate::DateTimeRange;

/// The lower bound of dates handled by specification
pub const DATE_START: NaiveDateTime = {
    let date = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
    let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    NaiveDateTime::new(date, time)
};

/// The upper bound of dates handled by specification
pub const DATE_END: NaiveDateTime = {
    let date = NaiveDate::from_ymd_opt(10_000, 1, 1).unwrap();
    let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    NaiveDateTime::new(date, time)
};

// OpeningHours

/// A parsed opening hours expression and its evaluation context.
///
/// Note that all big inner structures are immutable and wrapped by an `Arc`
/// so this is safe and fast to clone.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OpeningHours<L: Localize = NoLocation> {
    /// Rules describing opening hours
    expr: Arc<OpeningHoursExpression>,
    /// Evaluation context
    pub(crate) ctx: Context<L>,
}

impl OpeningHours<NoLocation> {
    /// Parse a raw opening hours expression.
    ///
    /// ```
    /// use opening_hours::{Context, OpeningHours};
    ///
    /// assert!(OpeningHours::parse("24/7 open").is_ok());
    /// assert!(OpeningHours::parse("not a valid expression").is_err());
    /// ```
    pub fn parse(raw_oh: &str) -> Result<Self, ParserError> {
        let expr = Arc::new(opening_hours_syntax::parse(raw_oh)?);
        Ok(Self { expr, ctx: Context::default() })
    }
}

impl<L: Localize> OpeningHours<L> {
    // --
    // -- Builder Methods
    // --

    /// Set a new evaluation context for this expression.
    ///
    /// ```
    /// use opening_hours::{Context, OpeningHours};
    ///
    /// let oh = OpeningHours::parse("Mo-Fr open")
    ///     .unwrap()
    ///     .with_context(Context::default());
    /// ```
    pub fn with_context<L2: Localize>(self, ctx: Context<L2>) -> OpeningHours<L2> {
        OpeningHours { expr: self.expr, ctx }
    }

    /// Convert the expression into a normalized form. It will not affect the meaning of the
    /// expression and might impact the performance of evaluations.
    ///
    /// ```
    /// use opening_hours::OpeningHours;
    ///
    /// let oh = OpeningHours::parse("24/7 ; Su closed").unwrap();
    /// assert_eq!(oh.normalize().to_string(), "Mo-Sa");
    /// ```
    pub fn normalize(&self) -> Self {
        Self {
            expr: Arc::new(self.expr.as_ref().clone().normalize()),
            ctx: self.ctx.clone(),
        }
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

    /// Get the schedule at a given day.
    pub fn schedule_at(&self, date: NaiveDate) -> Schedule {
        // eprintln!("Schedule at {date}");
        #[cfg(test)]
        crate::tests::stats::notify::generated_schedule();

        if !(DATE_START.date()..DATE_END.date()).contains(&date) {
            return Schedule::default();
        }

        let mut prev_match = false;
        let mut prev_eval = None;

        for rules_seq in &self.expr.rules {
            let curr_match = rules_seq.day_selector.filter(date, &self.ctx);
            let curr_eval = rule_sequence_schedule_at(rules_seq, date, &self.ctx);

            let (new_match, new_eval) = match (rules_seq.operator, rules_seq.kind) {
                // The normal rule acts like the additional rule when the kind is "closed".
                (RuleOperator::Normal, RuleKind::Open | RuleKind::Unknown) => (
                    curr_match || prev_match,
                    if curr_match {
                        curr_eval
                    } else {
                        prev_eval.or(curr_eval)
                    },
                ),
                (RuleOperator::Additional, _) | (RuleOperator::Normal, RuleKind::Closed) => (
                    prev_match || curr_match,
                    match (prev_eval, curr_eval) {
                        (Some(prev), Some(curr)) => Some(prev.addition(curr)),
                        (prev, curr) => prev.or(curr),
                    },
                ),
                (RuleOperator::Fallback, _) => {
                    if prev_match
                        && !(prev_eval.as_ref())
                            .map(Schedule::is_always_closed_with_no_comments)
                            .unwrap_or(false)
                    {
                        (prev_match, prev_eval)
                    } else {
                        (curr_match, curr_eval)
                    }
                }
            };

            prev_match = new_match;
            prev_eval = new_eval;
        }

        prev_eval
            .unwrap_or_else(Schedule::new)
            .filter_closed_ranges()
    }

    /// Same as [`iter_range`], but with naive date input and outputs.
    fn iter_range_naive(
        &self,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> impl Iterator<Item = DateTimeRange> + Send + Sync + use<L> {
        let from = std::cmp::min(DATE_END, from);
        let to = std::cmp::min(DATE_END, to);

        TimeDomainIterator::new(self, from, to)
            .take_while(move |dtr| dtr.range.start < to)
            .map(move |dtr| {
                let start = std::cmp::max(dtr.range.start, from);
                let end = std::cmp::min(dtr.range.end, to);

                DateTimeRange {
                    range: start..end,
                    kind: dtr.kind,
                    comment: dtr.comment.clone(),
                }
            })
    }

    // --
    // -- High level implementations / Syntactic sugar
    // --

    /// Iterate over disjoint intervals of different state restricted to the
    /// time interval `from..to`.
    pub fn iter_range(
        &self,
        from: L::DateTime,
        to: L::DateTime,
    ) -> impl Iterator<Item = DateTimeRange<L::DateTime>> + Send + Sync + use<L> {
        let locale = self.ctx.locale.clone();
        let naive_from = std::cmp::min(DATE_END, locale.naive(from));
        let naive_to = std::cmp::min(DATE_END, locale.naive(to));

        self.iter_range_naive(naive_from, naive_to)
            .map(move |dtr| DateTimeRange {
                range: locale.datetime(dtr.range.start)..locale.datetime(dtr.range.end),
                kind: dtr.kind,
                comment: dtr.comment.clone(),
            })
    }

    // Same as [`OpeningHours::iter_range`] but with an open end.
    pub fn iter_from(
        &self,
        from: L::DateTime,
    ) -> impl Iterator<Item = DateTimeRange<L::DateTime>> + Send + Sync + use<L> {
        self.iter_range(from, self.ctx.locale.datetime(DATE_END))
    }

    /// Get the next time where the state will change.
    ///
    /// ```
    /// use chrono::NaiveDateTime;
    /// use opening_hours::OpeningHours;
    /// use opening_hours_syntax::RuleKind;
    ///
    /// let oh = OpeningHours::parse("12:00-18:00 open, 18:00-20:00 unknown").unwrap();
    /// let date_1 = NaiveDateTime::parse_from_str("2024-11-18 15:00", "%Y-%m-%d %H:%M").unwrap();
    /// let date_2 = NaiveDateTime::parse_from_str("2024-11-18 18:00", "%Y-%m-%d %H:%M").unwrap();
    /// assert_eq!(oh.next_change(date_1), Some(date_2));
    /// ```
    pub fn next_change(&self, current_time: L::DateTime) -> Option<L::DateTime> {
        let interval = self.iter_from(current_time).next()?;

        if self.ctx.locale.naive(interval.range.end.clone()) >= DATE_END {
            None
        } else {
            Some(interval.range.end)
        }
    }

    /// Get the state at given time.
    ///
    /// ```
    /// use chrono::NaiveDateTime;
    /// use opening_hours::OpeningHours;
    /// use opening_hours_syntax::RuleKind;
    ///
    /// let oh = OpeningHours::parse("12:00-18:00 open, 18:00-20:00 unknown").unwrap();
    /// let date_1 = NaiveDateTime::parse_from_str("2024-11-18 15:00", "%Y-%m-%d %H:%M").unwrap();
    /// let date_2 = NaiveDateTime::parse_from_str("2024-11-18 19:00", "%Y-%m-%d %H:%M").unwrap();
    /// assert_eq!(oh.state(date_1), (RuleKind::Open, "".into()));
    /// assert_eq!(oh.state(date_2), (RuleKind::Unknown, "".into()));
    /// ```
    pub fn state(&self, current_time: L::DateTime) -> (RuleKind, Arc<str>) {
        self.iter_range(current_time.clone(), current_time + Duration::minutes(1))
            .next()
            .map(|dtr| dtr.into_state())
            .unwrap_or_default()
    }

    /// Check if this is open at a given time.
    ///
    /// ```
    /// use chrono::NaiveDateTime;
    /// use opening_hours::OpeningHours;
    ///
    /// let oh = OpeningHours::parse("12:00-18:00 open, 18:00-20:00 unknown").unwrap();
    /// let date_1 = NaiveDateTime::parse_from_str("2024-11-18 15:00", "%Y-%m-%d %H:%M").unwrap();
    /// let date_2 = NaiveDateTime::parse_from_str("2024-11-18 19:00", "%Y-%m-%d %H:%M").unwrap();
    /// assert!(oh.is_open(date_1));
    /// assert!(!oh.is_open(date_2));
    /// ```
    pub fn is_open(&self, current_time: L::DateTime) -> bool {
        self.state(current_time).0 == RuleKind::Open
    }

    /// Check if this is closed at a given time.
    ///
    /// ```
    /// use chrono::NaiveDateTime;
    /// use opening_hours::OpeningHours;
    ///
    /// let oh = OpeningHours::parse("12:00-18:00 open, 18:00-20:00 unknown").unwrap();
    /// let date_1 = NaiveDateTime::parse_from_str("2024-11-18 10:00", "%Y-%m-%d %H:%M").unwrap();
    /// let date_2 = NaiveDateTime::parse_from_str("2024-11-18 19:00", "%Y-%m-%d %H:%M").unwrap();
    /// assert!(oh.is_closed(date_1));
    /// assert!(!oh.is_closed(date_2));
    /// ```
    pub fn is_closed(&self, current_time: L::DateTime) -> bool {
        self.state(current_time).0 == RuleKind::Closed
    }

    /// Check if this is unknown at a given time.
    ///
    /// ```
    /// use chrono::NaiveDateTime;
    /// use opening_hours::OpeningHours;
    ///
    /// let oh = OpeningHours::parse("12:00-18:00 open, 18:00-20:00 unknown").unwrap();
    /// let date_1 = NaiveDateTime::parse_from_str("2024-11-18 19:00", "%Y-%m-%d %H:%M").unwrap();
    /// let date_2 = NaiveDateTime::parse_from_str("2024-11-18 15:00", "%Y-%m-%d %H:%M").unwrap();
    /// assert!(oh.is_unknown(date_1));
    /// assert!(!oh.is_unknown(date_2));
    /// ```
    pub fn is_unknown(&self, current_time: L::DateTime) -> bool {
        self.state(current_time).0 == RuleKind::Unknown
    }
}

impl FromStr for OpeningHours {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl<L: Localize> Display for OpeningHours<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)
    }
}

fn rule_sequence_schedule_at<L: Localize>(
    rule_sequence: &RuleSequence,
    date: NaiveDate,
    ctx: &Context<L>,
) -> Option<Schedule> {
    let from_today = Some(date)
        .filter(|date| rule_sequence.day_selector.filter(*date, ctx))
        .map(|date| time_selector_intervals_at(ctx, &rule_sequence.time_selector, date))
        .map(|rgs| Schedule::from_ranges(rgs, rule_sequence.kind, rule_sequence.comment.clone()));

    let from_yesterday = (date.pred_opt())
        .filter(|prev| rule_sequence.day_selector.filter(*prev, ctx))
        .map(|prev| time_selector_intervals_at_next_day(ctx, &rule_sequence.time_selector, prev))
        .map(|rgs| Schedule::from_ranges(rgs, rule_sequence.kind, rule_sequence.comment.clone()));

    match (from_today, from_yesterday) {
        (Some(sched_1), Some(sched_2)) => Some(sched_1.addition(sched_2)),
        (today, yesterday) => today.or(yesterday),
    }
}

// TimeDomainIterator

pub struct TimeDomainIterator<L: Clone + Localize> {
    opening_hours: OpeningHours<L>,
    curr_date: NaiveDate,
    curr_schedule: Peekable<crate::schedule::IntoIter>,
    end_datetime: NaiveDateTime,

    dates_with_schedule: Box<dyn Iterator<Item = NaiveDate> + Send + Sync + 'static>,
}

impl<L: Localize> TimeDomainIterator<L> {
    fn new(
        opening_hours: &OpeningHours<L>,
        start_datetime: NaiveDateTime,
        end_datetime: NaiveDateTime,
    ) -> Self {
        let opening_hours = opening_hours.clone();
        let start_date = start_datetime.date();
        let start_time = start_datetime.time().into();
        let mut curr_schedule = opening_hours.schedule_at(start_date).into_iter().peekable();

        let dates_with_schedule = Box::new(
            dates_with_potential_change(
                &opening_hours.expr,
                &opening_hours.ctx,
                start_datetime.date(),
                end_datetime.date(),
            ), // .inspect(|x| eprintln!("Jump to {x}")),
        );

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
            dates_with_schedule,
        }
    }

    fn consume_until_next_state(&mut self, curr_state: (RuleKind, &str)) {
        let start_date = self.curr_date;

        while self
            .curr_schedule
            .peek()
            .map(|tr| tr.as_state() == curr_state)
            .unwrap_or(false)
        {
            // Early return if infinite approximation is enabled
            if let Some(max_interval_size) = self.opening_hours.ctx.approx_bound_interval_size {
                if self.curr_date - start_date > max_interval_size + chrono::TimeDelta::days(1) {
                    return;
                }
            }

            self.curr_schedule.next();

            if self.curr_schedule.peek().is_none() {
                let mut next_change = self.dates_with_schedule.next().unwrap_or(DATE_END.date());

                while next_change <= self.curr_date {
                    next_change = self.dates_with_schedule.next().unwrap_or(DATE_END.date());
                }

                // let next_change_hint = self
                //     .opening_hours
                //     .next_change_hint(self.curr_date)
                //     .unwrap_or_else(|| self.curr_date.succ_opt().expect("reached invalid date"));
                //
                // assert!(next_change_hint > self.curr_date, "infinite loop detected");
                self.curr_date = next_change;

                if self.curr_date > self.end_datetime.date() || self.curr_date >= DATE_END.date() {
                    break;
                }

                self.curr_schedule = (self.opening_hours)
                    .schedule_at(self.curr_date)
                    .into_iter()
                    .peekable();
            }
        }
    }
}

impl<L: Localize> Iterator for TimeDomainIterator<L> {
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

            self.consume_until_next_state(curr_tr.as_state());
            let end_date = self.curr_date;

            let end_time = self
                .curr_schedule
                .peek()
                .map(|tr| tr.range.start)
                .unwrap_or(ExtendedTime::MIDNIGHT_00);

            let end = std::cmp::min(
                self.end_datetime,
                NaiveDateTime::new(
                    end_date,
                    end_time.try_into().expect("got invalid time from schedule"),
                ),
            );

            // Infinity approximation, if enabled
            if let Some(max_interval_size) = self.opening_hours.ctx.approx_bound_interval_size {
                if end - start > max_interval_size {
                    return Some(DateTimeRange {
                        range: start..DATE_END,
                        kind: curr_tr.kind,
                        comment: curr_tr.comment.clone(),
                    });
                }
            }

            Some(DateTimeRange {
                range: start..end,
                kind: curr_tr.kind,
                comment: curr_tr.comment.clone(),
            })
        } else {
            None
        }
    }
}
