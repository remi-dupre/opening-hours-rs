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

#[derive(Clone, Debug)]
pub struct Selector {
    pub year: YearSelector,
    pub monthday: MonthdaySelector,
    pub week: WeekSelector,
    pub time: TimeSelector,
}

impl Selector {
    pub fn always_open() -> Self {
        Self {
            year: YearSelector::range(1900, 9999),
            monthday: MonthdaySelector::month_range(Month::January, Month::December),
            week: WeekSelector::range(1, 53),
            time: TimeSelector::range(
                NaiveTime::from_hms(0, 0, 0),
                NaiveTime::from_hms(23, 59, 59),
            ),
        }
    }
}

// ---
// --- Year selector
// ---

#[derive(Clone, Debug)]
pub struct YearSelector(pub Vec<YearRange>);

impl YearSelector {
    pub fn range(start: u16, end: u16) -> Self {
        assert!(1900 <= start && start <= end && end <= 9999);

        Self(vec![YearRange {
            start,
            end,
            step: 1,
        }])
    }
}

#[derive(Clone, Debug)]
pub struct YearRange {
    pub start: u16,
    pub end: u16,
    pub step: u16,
}

// ---
// --- Monthday selector
// ---

#[derive(Clone, Debug)]
pub struct MonthdaySelector(pub Vec<MonthdayRange>);

impl MonthdaySelector {
    pub fn month_range(start: Month, end: Month) -> Self {
        assert!(start <= end);

        Self(vec![MonthdayRange::Month {
            year: None,
            start,
            end,
        }])
    }
}

#[derive(Clone, Debug)]
pub enum MonthdayRange {
    Month {
        year: Option<u16>,
        start: Month,
        end: Month,
    },
    Date {
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

#[derive(Clone, Debug)]
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
// --- Week selector
// ---

#[derive(Clone, Debug)]
pub struct WeekSelector(Vec<WeekRange>);

impl WeekSelector {
    pub fn new(ranges: impl IntoIterator<Item = WeekRange>) -> Self {
        Self(ranges.into_iter().collect())
    }

    pub fn range(start: u8, end: u8) -> Self {
        assert!(1 <= start && start <= end && end <= 53);

        Self(vec![WeekRange {
            start,
            end,
            step: 1,
        }])
    }
}

#[derive(Clone, Debug)]
pub struct WeekRange {
    pub start: u8,
    pub end: u8,
    pub step: u8,
}

// ---
// --- Day selector
// ---

#[derive(Clone, Debug)]
pub enum Day {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}

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
pub struct TimeSelector(pub Vec<TimeSpan>);

impl TimeSelector {
    pub fn range(start: NaiveTime, end: NaiveTime) -> Self {
        Self(vec![TimeSpan {
            start: Time::Fixed(start),
            end: Time::Fixed(end),
            repeats: None,
            open_end: false,
        }])
    }
}

#[derive(Clone, Debug)]
pub struct TimeSpan {
    pub start: Time,
    pub end: Time,
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
