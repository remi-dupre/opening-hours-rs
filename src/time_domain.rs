use std::boxed::Box;
use std::cmp::{max, min};
use std::convert::TryInto;
use std::fmt;
use std::iter::{empty, Peekable};
use std::ops::Range;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use once_cell::sync::Lazy;

use crate::day_selector::{DateFilter, DaySelector};
use crate::extended_time::ExtendedTime;
use crate::schedule::{Schedule, TimeRange};
use crate::time_selector::TimeSelector;

static DATE_LIMIT: Lazy<NaiveDateTime> = Lazy::new(|| {
    NaiveDateTime::new(
        NaiveDate::from_ymd(10_000, 1, 1),
        NaiveTime::from_hms(0, 0, 0),
    )
});

// DateTimeRange

#[non_exhaustive]
#[derive(Clone)]
pub struct DateTimeRange<'c> {
    pub range: Range<NaiveDateTime>,
    pub kind: RuleKind,
    pub comments: Vec<&'c str>,
}

impl fmt::Debug for DateTimeRange<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DateTimeRange")
            .field("range", &self.range)
            .field("kind", &self.kind)
            .field("comments", &self.comments)
            .finish()
    }
}

impl<'c> DateTimeRange<'c> {
    fn new_with_sorted_comments(
        range: Range<NaiveDateTime>,
        kind: RuleKind,
        comments: Vec<&'c str>,
    ) -> Self {
        Self {
            range,
            kind,
            comments,
        }
    }
}

// TimeDomain

#[derive(Clone, Debug)]
pub struct TimeDomain {
    pub rules: Vec<RuleSequence>,
}

impl TimeDomain {
    // Low level implementations.
    //
    // Following functions are used to build the TimeDomainIterator which is
    // used to implement all other functions.
    //
    // This means that performances matters a lot for these functions and it
    // would be relevant to focus on optimisatons to this regard.

    pub fn schedule_at(&self, date: NaiveDate) -> Schedule {
        let mut prev_match = false;
        let mut prev_eval = None;

        for rules_seq in &self.rules {
            let curr_match = rules_seq.day_selector.filter(date);
            let curr_eval = rules_seq.schedule_at(date);

            let (new_match, new_eval) = match rules_seq.operator {
                RuleOperator::Normal => {
                    if prev_match {
                        return prev_eval.expect("schedule can't be None for matching days");
                    } else {
                        (curr_match, curr_eval)
                    }
                }
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

    pub fn iter_range(&self, from: NaiveDateTime, to: NaiveDateTime) -> TimeDomainIterator {
        assert!(to <= *DATE_LIMIT);
        TimeDomainIterator::new(self, from, to)
    }

    pub fn iter_from(&self, from: NaiveDateTime) -> TimeDomainIterator {
        self.iter_range(from, *DATE_LIMIT)
    }

    // High level implementations

    pub fn next_change(&self, current_time: NaiveDateTime) -> NaiveDateTime {
        self.iter_from(current_time)
            .next()
            .map(|dtr| dtr.range.end)
            .unwrap_or(current_time)
    }

    pub fn state(&self, current_time: NaiveDateTime) -> RuleKind {
        self.iter_range(current_time, current_time + Duration::minutes(1))
            .next()
            .map(|dtr| dtr.kind)
            .unwrap_or(RuleKind::Unknown)
    }

    pub fn is_open(&self, current_time: NaiveDateTime) -> bool {
        self.state(current_time) == RuleKind::Open
    }

    pub fn is_closed(&self, current_time: NaiveDateTime) -> bool {
        self.state(current_time) == RuleKind::Closed
    }

    pub fn is_unknown(&self, current_time: NaiveDateTime) -> bool {
        self.state(current_time) == RuleKind::Unknown
    }

    pub fn intervals<'s>(
        &'s self,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> impl Iterator<Item = DateTimeRange> + 's {
        self.iter_from(from)
            .take_while(move |dtr| dtr.range.start < to)
            .map(move |dtr| {
                let start = max(dtr.range.start, from);
                let end = min(dtr.range.end, to);
                DateTimeRange::new_with_sorted_comments(start..end, dtr.kind, dtr.comments)
            })
    }
}

// TimeDomainIterator

pub struct TimeDomainIterator<'d> {
    time_domain: &'d TimeDomain,
    curr_date: NaiveDate,
    curr_schedule: Peekable<Box<dyn Iterator<Item = TimeRange<'d>> + 'd>>,
    end_datetime: NaiveDateTime,
}

impl<'d> TimeDomainIterator<'d> {
    pub fn new(
        time_domain: &'d TimeDomain,
        start_datetime: NaiveDateTime,
        end_datetime: NaiveDateTime,
    ) -> Self {
        let start_date = start_datetime.date();
        let start_time = start_datetime.time().into();

        let mut curr_schedule = {
            if start_datetime < end_datetime {
                time_domain.schedule_at(start_date).into_iter_filled()
            } else {
                Box::new(empty())
            }
        }
        .peekable();

        while curr_schedule
            .peek()
            .map(|tr| !tr.range.contains(&start_time))
            .unwrap_or(false)
        {
            curr_schedule.next();
        }

        Self {
            time_domain,
            curr_date: start_date,
            curr_schedule,
            end_datetime,
        }
    }

    fn consume_until_next_kind(&mut self, curr_kind: RuleKind) {
        while self.curr_schedule.peek().map(|tr| tr.kind) == Some(curr_kind) {
            self.curr_schedule.next();

            if self.curr_schedule.peek().is_none() {
                self.curr_date += Duration::days(1);

                if self.curr_date <= self.end_datetime.date() {
                    self.curr_schedule = self
                        .time_domain
                        .schedule_at(self.curr_date)
                        .into_iter_filled()
                        .peekable()
                }
            }
        }
    }
}

impl<'d> Iterator for TimeDomainIterator<'d> {
    type Item = DateTimeRange<'d>;

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

// RuleSequence

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct RuleSequence {
    pub day_selector: DaySelector,
    pub time_selector: TimeSelector,
    pub kind: RuleKind,
    pub operator: RuleOperator,
    pub comments: Vec<String>,
}

impl RuleSequence {
    pub fn new(
        day_selector: DaySelector,
        time_selector: TimeSelector,
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
}

impl RuleSequence {
    pub fn schedule_at(&self, date: NaiveDate) -> Option<Schedule> {
        let today = {
            if self.day_selector.filter(date) {
                let ranges = self.time_selector.intervals_at(date);
                Some(Schedule::from_ranges_with_sorted_comments(
                    ranges,
                    self.kind,
                    self.get_comments(),
                ))
            } else {
                None
            }
        };

        let yesterday = {
            let date = date - Duration::days(1);

            if self.day_selector.filter(date) {
                let ranges = self.time_selector.intervals_at_next_day(date);
                Some(Schedule::from_ranges_with_sorted_comments(
                    ranges,
                    self.kind,
                    self.get_comments(),
                ))
            } else {
                None
            }
        };

        match (today, yesterday) {
            (Some(sched_1), Some(sched_2)) => Some(sched_1.addition(sched_2)),
            (today, yesterday) => today.or(yesterday),
        }
    }

    pub fn get_comments(&self) -> Vec<&str> {
        self.comments.iter().map(|x| x.as_str()).collect()
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
