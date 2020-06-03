use std::boxed::Box;
use std::cmp::{max, min};
use std::convert::TryInto;
use std::iter::Peekable;
use std::ops::{Range, RangeInclusive};

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate, NaiveDateTime};

use crate::extended_time::ExtendedTime;
use crate::schedule::Schedule;
use crate::time_selector::DateFilter;
use crate::utils::time_ranges_union;

pub type Weekday = chrono::Weekday;

// TimeDomain

#[derive(Clone, Debug)]
pub struct TimeDomain {
    // TODO: handle additional rule
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
        // TODO: handle "additional rule"
        // TODO: handle comments
        self.rules
            .iter()
            .filter(|rules_seq| rules_seq.feasible_date(date))
            .last()
            .map(|rules_seq| rules_seq.schedule_at(date))
            .unwrap_or_default()
    }

    pub fn feasible_date(&self, date: NaiveDate) -> bool {
        self.rules
            .iter()
            .any(|rules_seq| rules_seq.feasible_date(date))
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

    pub fn state(&self, current_time: NaiveDateTime) -> RulesModifier {
        self.iter_from(current_time)
            .next()
            .map(|(_, state)| state)
            .unwrap_or(RulesModifier::Unknown)
    }

    pub fn is_open(&self, current_time: NaiveDateTime) -> bool {
        self.state(current_time) == RulesModifier::Open
    }

    pub fn is_closed(&self, current_time: NaiveDateTime) -> bool {
        self.state(current_time) == RulesModifier::Closed
    }

    pub fn is_unknown(&self, current_time: NaiveDateTime) -> bool {
        self.state(current_time) == RulesModifier::Unknown
    }

    pub fn intervals<'s>(
        &'s self,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> impl Iterator<Item = (Range<NaiveDateTime>, RulesModifier)> + 's {
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
    curr_schedule: Peekable<Box<dyn Iterator<Item = (Range<ExtendedTime>, RulesModifier)>>>,
}

impl<'d> TimeDomainIterator<'d> {
    pub fn new(time_domain: &'d TimeDomain, start_datetime: NaiveDateTime) -> Self {
        let start_date = start_datetime.date();
        let start_time = start_datetime.time().into();

        let mut curr_schedule = {
            if start_date.year() <= 9999 {
                time_domain.schedule_at(start_date).into_iter_with_closed()
            } else {
                Box::new(std::iter::empty())
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

    fn consume_until_next_state(&mut self, curr_state: RulesModifier) {
        while self.curr_schedule.peek().map(|(_, st)| *st) == Some(curr_state) {
            self.curr_schedule.next();

            if self.curr_schedule.peek().is_none() {
                self.curr_date += Duration::days(1);

                if self.curr_date.year() <= 9999 {
                    self.curr_schedule = self
                        .time_domain
                        .schedule_at(self.curr_date)
                        .into_iter_with_closed()
                        .peekable()
                }
            }
        }
    }
}

impl Iterator for TimeDomainIterator<'_> {
    type Item = (Range<NaiveDateTime>, RulesModifier);

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
    pub selector: Selector,
    pub modifier: RulesModifier,
    pub comment: Option<String>,
}

impl RuleSequence {
    pub fn feasible_date(&self, date: NaiveDate) -> bool {
        self.selector.feasible_date(date)
    }

    pub fn schedule_at(&self, date: NaiveDate) -> Schedule {
        let ranges = self.selector.intervals_at(date);
        Schedule::from_ranges(ranges, self.modifier)
    }
}

// RulesModifier

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RulesModifier {
    Closed,
    Open,
    Unknown,
}

// Selector

#[derive(Clone, Debug, Default)]
pub struct Selector {
    pub year: Vec<YearRange>,
    pub monthday: Vec<MonthdayRange>,
    pub week: Vec<WeekRange>,
    pub weekday: Vec<WeekDayRange>,
    pub time: Vec<TimeSpan>,
}

impl Selector {
    pub fn intervals_at(&self, date: NaiveDate) -> Vec<Range<ExtendedTime>> {
        time_ranges_union(self.time.iter().map(|span| span.as_naive_time(date)))
    }

    // TODO: this should be private
    pub fn feasible_date(&self, date: NaiveDate) -> bool {
        Self::check_date_field(&self.year, date)
            && Self::check_date_field(&self.monthday, date)
            && Self::check_date_field(&self.week, date)
            && Self::check_date_field(&self.weekday, date)
    }

    fn check_date_field<T: DateFilter>(selector_field: &[T], date: NaiveDate) -> bool {
        selector_field.is_empty() || selector_field.iter().any(|x| x.filter(date))
    }
}

// ---
// --- Year selector
// ---

// YearRange

#[derive(Clone, Debug)]
pub struct YearRange {
    pub range: RangeInclusive<u16>,
    pub step: u16,
}

// ---
// --- Monthday selector
// ---

#[derive(Clone, Debug)]
pub enum MonthdayRange {
    Month {
        range: RangeInclusive<Month>,
        year: Option<u16>,
    },
    Date {
        start: (Date, DateOffset),
        end: (Date, DateOffset),
    },
}

// Date

#[derive(Clone, Copy, Debug)]
pub enum Date {
    Fixed {
        year: Option<u16>,
        month: Month,
        day: u8,
    },
    Easter {
        year: Option<u16>,
    },
}

impl Date {
    pub fn day(day: u8, month: Month, year: u16) -> Self {
        Self::Fixed {
            day,
            month,
            year: Some(year),
        }
    }
}

// DateOffset

#[derive(Clone, Debug, Default)]
pub struct DateOffset {
    pub wday_offset: WeekDayOffset,
    pub day_offset: i64,
}

impl DateOffset {
    pub fn apply(&self, mut date: NaiveDate) -> NaiveDate {
        date += Duration::days(self.day_offset);

        match self.wday_offset {
            WeekDayOffset::None => {}
            WeekDayOffset::Prev(target) => {
                while date.weekday() != target {
                    date -= Duration::days(1);
                }
            }
            WeekDayOffset::Next(target) => {
                while date.weekday() != target {
                    date += Duration::days(1);
                }
            }
        }

        date
    }
}

// WeekDayOffset

#[derive(Clone, Copy, Debug)]
pub enum WeekDayOffset {
    None,
    Next(Weekday),
    Prev(Weekday),
}

impl Default for WeekDayOffset {
    fn default() -> Self {
        Self::None
    }
}

// ---
// --- WeekDay selector
// ---

// WeekDayRange

#[derive(Clone, Debug)]
pub enum WeekDayRange {
    Fixed {
        range: RangeInclusive<Weekday>,
        nth: Vec<u8>, // TODO: maybe a tiny bitset would make more sense
        offset: i64,
    },
    Holiday {
        kind: HolidayKind,
        offset: i64,
    },
}

// HolidayKind

#[derive(Clone, Copy, Debug)]
pub enum HolidayKind {
    Public,
    School,
}

// ---
// --- Week selector
// ---

// Week selector

#[derive(Clone, Debug)]
pub struct WeekRange {
    pub range: RangeInclusive<u8>,
    pub step: u8,
}

// ---
// --- Day selector
// ---

// Month

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl Month {
    pub fn from_u8(x: u8) -> Result<Self, ()> {
        Ok(match x {
            1 => Self::January,
            2 => Self::February,
            3 => Self::March,
            4 => Self::April,
            5 => Self::May,
            6 => Self::June,
            7 => Self::July,
            8 => Self::August,
            9 => Self::September,
            10 => Self::October,
            11 => Self::November,
            12 => Self::December,
            _ => return Err(()),
        })
    }

    pub fn next(self) -> Self {
        let num = self as u8;
        Self::from_u8((num % 12) + 1).unwrap()
    }
}

// ---
// --- Time selector
// ---

// TimeSpan

#[derive(Clone, Debug)]
pub struct TimeSpan {
    pub range: Range<Time>,
    pub open_end: bool,
    pub repeats: Option<Duration>,
}

impl TimeSpan {
    pub fn as_naive_time(&self, date: NaiveDate) -> Range<ExtendedTime> {
        let start = self.range.start.as_naive(date);
        let end = self.range.end.as_naive(date);
        start..end
    }
}

// Time

#[derive(Copy, Clone, Debug)]
pub enum Time {
    Fixed(ExtendedTime),
    Variable(VariableTime),
}

impl Time {
    pub fn as_naive(self, date: NaiveDate) -> ExtendedTime {
        match self {
            Time::Fixed(naive) => naive,
            Time::Variable(variable) => variable.as_naive(date),
        }
    }
}

// VariableTime

#[derive(Copy, Clone, Debug)]
pub struct VariableTime {
    pub event: TimeEvent,
    pub offset: i16,
}

impl VariableTime {
    pub fn as_naive(self, date: NaiveDate) -> ExtendedTime {
        self.event
            .as_naive(date)
            .add_minutes(self.offset)
            .unwrap_or_else(|_| ExtendedTime::new(0, 0))
    }
}

// TimeEvent

#[derive(Clone, Copy, Debug)]
pub enum TimeEvent {
    Dawn,
    Sunrise,
    Sunset,
    Dusk,
}

impl TimeEvent {
    pub fn as_naive(self, _date: NaiveDate) -> ExtendedTime {
        // TODO: real computation based on the day (and position/timezone?)
        match self {
            Self::Dawn => ExtendedTime::new(6, 0),
            Self::Sunrise => ExtendedTime::new(7, 0),
            Self::Sunset => ExtendedTime::new(19, 0),
            Self::Dusk => ExtendedTime::new(18, 0),
        }
    }
}
