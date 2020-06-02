use std::ops::{Range, RangeInclusive};

use chrono::prelude::Datelike;
use chrono::Duration;
use chrono::NaiveDate;

use crate::extended_time::ExtendedTime;
use crate::schedule::Schedule;
use crate::time_selector::DateFilter;
use crate::utils::time_ranges_union;

pub type Weekday = chrono::Weekday;

// TimeDomain

#[derive(Clone, Debug)]
pub struct TimeDomain {
    // TODO: handle additional rule
    // TODO: use internal time repr
    pub rules: Vec<RuleSequence>,
}

impl TimeDomain {
    pub fn schedule_at(&self, date: NaiveDate) -> Schedule {
        // TODO: handle "additional rule"
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
