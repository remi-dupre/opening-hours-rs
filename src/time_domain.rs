use std::boxed::Box;
use std::cmp::{max, min};
use std::convert::TryInto;
use std::iter::{empty, Peekable};
use std::ops::Range;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate, NaiveDateTime};

use crate::day_selector::{DateFilter, DaySelector};
use crate::extended_time::ExtendedTime;
use crate::schedule::Schedule;
use crate::time_selector::TimeSelector;

pub type Weekday = chrono::Weekday;

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
        // TODO: handle comments
        self.rules
            .iter()
            .fold(None, |prev_eval, rules_seq| {
                let curr_eval = rules_seq.schedule_at(date);

                match rules_seq.operator {
                    RuleOperator::Normal => curr_eval,
                    RuleOperator::Additional => match (prev_eval, curr_eval) {
                        (Some(prev), Some(curr)) => Some(prev.addition(curr)),
                        (prev, curr) => prev.or(curr),
                    },
                    RuleOperator::Fallback => prev_eval.or(curr_eval),
                }
            })
            .unwrap_or_else(Schedule::empty)
    }

    pub fn iter_from(&self, from: NaiveDateTime) -> TimeDomainIterator {
        TimeDomainIterator::new(self, from)
    }

    // High level implementations

    pub fn next_change(&self, current_time: NaiveDateTime) -> NaiveDateTime {
        self.iter_from(current_time)
            .next()
            .map(|(range, _)| range.end)
            .unwrap_or(current_time)
    }

    pub fn state(&self, current_time: NaiveDateTime) -> RuleKind {
        self.iter_from(current_time)
            .next()
            .map(|(_, state)| state)
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
    ) -> impl Iterator<Item = (Range<NaiveDateTime>, RuleKind)> + 's {
        self.iter_from(from)
            .take_while(move |(range, _)| range.start < to)
            .map(move |(range, state)| {
                let start = max(range.start, from);
                let end = min(range.end, to);
                (start..end, state)
            })
    }
}

// TimeDomainIterator

pub struct TimeDomainIterator<'d> {
    time_domain: &'d TimeDomain,
    curr_date: NaiveDate,
    curr_schedule: Peekable<Box<dyn Iterator<Item = (Range<ExtendedTime>, RuleKind)>>>,
}

impl<'d> TimeDomainIterator<'d> {
    pub fn new(time_domain: &'d TimeDomain, start_datetime: NaiveDateTime) -> Self {
        let start_date = start_datetime.date();
        let start_time = start_datetime.time().into();

        let mut curr_schedule = {
            if start_date.year() <= 9999 {
                time_domain.schedule_at(start_date).into_iter_filled()
            } else {
                Box::new(empty())
            }
        }
        .peekable();

        while curr_schedule
            .peek()
            .map(|(range, _)| !range.contains(&start_time))
            .unwrap_or(false)
        {
            curr_schedule.next();
        }

        Self {
            time_domain,
            curr_date: start_date,
            curr_schedule,
        }
    }

    fn consume_until_next_state(&mut self, curr_state: RuleKind) {
        while self.curr_schedule.peek().map(|(_, st)| *st) == Some(curr_state) {
            self.curr_schedule.next();

            if self.curr_schedule.peek().is_none() {
                self.curr_date += Duration::days(1);

                if self.curr_date.year() <= 9999 {
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

impl Iterator for TimeDomainIterator<'_> {
    type Item = (Range<NaiveDateTime>, RuleKind);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((curr_range, curr_state)) = self.curr_schedule.peek().cloned() {
            let start = NaiveDateTime::new(
                self.curr_date,
                curr_range
                    .start
                    .try_into()
                    .expect("got invalid time from schedule"),
            );

            self.consume_until_next_state(curr_state);

            let end_date = self.curr_date;
            let end_time = self
                .curr_schedule
                .peek()
                .map(|(range, _)| range.start)
                .unwrap_or_else(|| ExtendedTime::new(0, 0));

            let end = NaiveDateTime::new(
                end_date,
                end_time.try_into().expect("got invalid time from schedule"),
            );

            Some((start..end, curr_state))
        } else {
            None
        }
    }
}

// RuleSequence

#[derive(Clone, Debug)]
pub struct RuleSequence {
    pub day_selector: DaySelector,
    pub time_selector: TimeSelector,
    pub kind: RuleKind,
    pub operator: RuleOperator,
    pub comments: Vec<String>,
}

impl RuleSequence {
    pub fn schedule_at(&self, date: NaiveDate) -> Option<Schedule> {
        let today = {
            if self.day_selector.filter(date) {
                let ranges = self.time_selector.intervals_at(date);
                Some(Schedule::from_ranges(ranges, self.kind))
            } else {
                None
            }
        };

        let yesterday = {
            let date = date - Duration::days(1);

            if self.day_selector.filter(date) {
                let ranges = self.time_selector.intervals_at_next_day(date);
                Some(Schedule::from_ranges(ranges, self.kind))
            } else {
                None
            }
        };

        match (today, yesterday) {
            (Some(sched_1), Some(sched_2)) => Some(sched_1.addition(sched_2)),
            (today, yesterday) => today.or(yesterday),
        }
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
