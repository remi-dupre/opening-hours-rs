use std::ops::RangeInclusive;
use std::time::Duration;

use chrono::NaiveTime;

#[derive(Clone, Debug)]
pub struct TimeDomain {
    pub rules: Vec<RuleSequence>,
}

#[derive(Clone, Debug)]
pub struct RuleSequence {
    pub selector: Selector,
    pub modifier: RulesModifier,
    pub comment: Option<String>,
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
    pub weekday: Vec<WeekdayRange>,
    pub time: Vec<TimeSpan>,
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
        year: Option<u16>,
        start: Month,
        end: Month,
    },
    Date {
        // TODO: merge DateFrom and DateTo types (and use a RangeInclusive?)
        start: (DateFrom, DateOffset),
        end: (DateTo, DateOffset),
    },
}

#[derive(Clone, Debug)]
pub enum DateFrom {
    Fixed {
        year: Option<u16>,
        month: Month,
        day: u8,
    },
    Easter {
        year: Option<u16>,
    },
}

impl DateFrom {
    pub fn day(day: u8, month: Month, year: u16) -> Self {
        Self::Fixed {
            day,
            month,
            year: Some(year),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DateTo {
    DateFrom(DateFrom),
    DayNum(u8),
}

impl DateTo {
    pub fn day(day: u8, month: Month, year: u16) -> Self {
        Self::DateFrom(DateFrom::day(day, month, year))
    }
}

#[derive(Clone, Debug)]
pub struct DateOffset {
    pub wday_offset: WeekDayOffset,
    pub day_offset: i64,
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

#[derive(Clone, Copy, Debug)]
pub enum Weekday {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

// ---
// --- Weekday selector
// ---

#[derive(Clone, Debug)]
pub enum WeekdayRange {
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
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
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
