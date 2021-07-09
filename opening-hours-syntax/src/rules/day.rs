use std::convert::{TryFrom, TryInto};
use std::ops::RangeInclusive;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate};

// Reexport Weekday from chrono as part of the public type.
pub use chrono::Weekday;

// Errors

#[derive(Debug)]
pub struct InvalidMonth;

// DaySelector

#[derive(Clone, Debug, Default)]
pub struct DaySelector {
    pub year: Vec<YearRange>,
    pub monthday: Vec<MonthdayRange>,
    pub week: Vec<WeekRange>,
    pub weekday: Vec<WeekDayRange>,
}

// YearRange

#[derive(Clone, Debug)]
pub struct YearRange {
    pub range: RangeInclusive<u16>,
    pub step: u16,
}

// MonthdayRange

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
    #[inline]
    pub fn day(day: u8, month: Month, year: u16) -> Self {
        Self::Fixed { day, month, year: Some(year) }
    }
}

// DateOffset

#[derive(Clone, Debug, Default)]
pub struct DateOffset {
    pub wday_offset: WeekDayOffset,
    pub day_offset: i64,
}

impl DateOffset {
    #[inline]
    pub fn apply(&self, mut date: NaiveDate) -> NaiveDate {
        date += Duration::days(self.day_offset);

        match self.wday_offset {
            WeekDayOffset::None => {}
            WeekDayOffset::Prev(target) => {
                let diff = (7 + target as i64 - date.weekday() as i64) % 7;
                date -= Duration::days(diff)
            }
            WeekDayOffset::Next(target) => {
                let diff = (7 + date.weekday() as i64 - target as i64) % 7;
                date += Duration::days(diff)
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
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

// WeekDayRange

#[derive(Clone, Debug)]
pub enum WeekDayRange {
    Fixed {
        range: RangeInclusive<Weekday>,
        offset: i64,
        nth: [bool; 5],
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

// WeekRange

#[derive(Clone, Debug)]
pub struct WeekRange {
    pub range: RangeInclusive<u8>,
    pub step: u8,
}

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
    #[inline]
    pub fn next(self) -> Self {
        let num = self as u8;
        ((num % 12) + 1).try_into().unwrap()
    }
}

macro_rules! impl_try_into_for_month {
    ( $from_type: ty ) => {
        impl TryFrom<$from_type> for Month {
            type Error = InvalidMonth;

            #[inline]
            fn try_from(value: $from_type) -> Result<Self, Self::Error> {
                let value: u8 = value.try_into().map_err(|_| InvalidMonth)?;

                Ok(match value {
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
                    _ => return Err(InvalidMonth),
                })
            }
        }
    };
    ( $from_type: ty, $( $tail: tt )+ ) => {
        impl_try_into_for_month!($from_type);
        impl_try_into_for_month!($($tail)+);
    };
}

impl_try_into_for_month!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);
