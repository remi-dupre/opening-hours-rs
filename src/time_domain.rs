use std::ops::RangeInclusive;

use chrono::prelude::Datelike;
use chrono::Duration;
use chrono::{NaiveDate, NaiveTime};

use crate::time_selector::DateFilter;

pub type Weekday = chrono::Weekday;

#[derive(Clone, Debug)]
pub struct TimeDomain {
    pub rules: Vec<RuleSequence>,
}

impl TimeDomain {
    pub fn feasible_date(&self, date: NaiveDate) -> bool {
        self.rules[0].feasible_date(date)
    }
}

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
}

#[derive(Clone, Debug)]
pub enum RulesModifier {
    Closed,
    Open,
    Unknown,
}

#[derive(Clone, Debug, Default)]
pub struct Selector {
    pub year: Vec<YearRange>,
    pub monthday: Vec<MonthdayRange>,
    pub week: Vec<WeekRange>,
    pub weekday: Vec<WeekDayRange>,
    pub time: Vec<TimeSpan>,
}

fn check_date_selector_field<T: DateFilter>(selector_field: &[T], date: NaiveDate) -> bool {
    selector_field.is_empty() || selector_field.iter().any(|x| x.filter(date))
}

impl Selector {
    // TODO: this should be private
    pub fn feasible_date(&self, date: NaiveDate) -> bool {
        check_date_selector_field(&self.year, date)
            && check_date_selector_field(&self.monthday, date)
            && check_date_selector_field(&self.week, date)
            && check_date_selector_field(&self.weekday, date)
    }
}

// ---
// --- Year selector
// ---

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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

impl Default for DateOffset {
    fn default() -> Self {
        Self {
            wday_offset: WeekDayOffset::None,
            day_offset: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum WeekDayOffset {
    None,
    Next(Weekday),
    Prev(Weekday),
}

// ---
// --- WeekDay selector
// ---

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

#[derive(Clone, Debug)]
pub enum HolidayKind {
    Public,
    School,
}

// ---
// --- Week selector
// ---

#[derive(Clone, Debug)]
pub struct WeekRange {
    pub range: RangeInclusive<u8>,
    pub step: u8,
}

// ---
// --- Day selector
// ---

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

#[derive(Clone, Debug)]
pub struct TimeSpan {
    pub range: RangeInclusive<Time>,
    pub open_end: bool,
    pub repeats: Option<Duration>,
}

#[derive(Clone, Debug)]
pub enum Time {
    Fixed(NaiveTime),
    Variable(VariableTime),
}

#[derive(Clone, Debug)]
pub struct VariableTime {
    pub event: TimeEvent,
    pub offset: i16,
}

#[derive(Clone, Debug)]
pub enum TimeEvent {
    Dawn,
    Sunrise,
    Sunset,
    Dusk,
}
