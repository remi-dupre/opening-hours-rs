use std::ops::RangeInclusive;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate};

pub type Weekday = chrono::Weekday;

/// Generic trait to specify the behavior of a selector over dates.
pub trait DateFilter {
    fn filter(&self, date: NaiveDate) -> bool;
}

impl<T: DateFilter> DateFilter for [T] {
    fn filter(&self, date: NaiveDate) -> bool {
        self.is_empty() || self.iter().any(|x| x.filter(date))
    }
}

// DaySelector

#[derive(Clone, Debug, Default)]
pub struct DaySelector {
    pub year: Vec<YearRange>,
    pub monthday: Vec<MonthdayRange>,
    pub week: Vec<WeekRange>,
    pub weekday: Vec<WeekDayRange>,
}

impl DateFilter for DaySelector {
    fn filter(&self, date: NaiveDate) -> bool {
        self.year.filter(date)
            && self.monthday.filter(date)
            && self.week.filter(date)
            && self.weekday.filter(date)
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

impl DateFilter for YearRange {
    fn filter(&self, date: NaiveDate) -> bool {
        let year = date.year() as u16;
        self.range.contains(&year) && (year - self.range.start()) % self.step == 0
    }
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

impl DateFilter for MonthdayRange {
    fn filter(&self, date: NaiveDate) -> bool {
        let in_year = date.year() as u16;
        let in_month = Month::from_u8(date.month() as u8).expect("invalid month value");

        match self {
            MonthdayRange::Month { year, range } => {
                year.unwrap_or(in_year) == in_year && range.contains(&in_month)
            }
            MonthdayRange::Date {
                start: (start, start_offset),
                end: (end, end_offset),
            } => match (&start, end) {
                (
                    Date::Fixed {
                        year: year_1,
                        month: month_1,
                        day: day_1,
                    },
                    Date::Fixed {
                        year: year_2,
                        month: month_2,
                        day: day_2,
                    },
                ) => {
                    let start = NaiveDate::from_ymd(
                        year_1.unwrap_or(date.year() as u16) as i32,
                        *month_1 as u32,
                        *day_1 as u32,
                    );

                    let mut end = NaiveDate::from_ymd(
                        year_2.unwrap_or(date.year() as u16) as i32,
                        *month_2 as u32,
                        *day_2 as u32,
                    );

                    if end < start {
                        end = end.with_year(end.year() + 1).expect("year overflow")
                    }

                    let start = start_offset.apply(start);
                    let end = end_offset.apply(end);
                    (start..=end).contains(&date)
                }
                _ => todo!("Easter not implemented yet"),
            },
        }
    }
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

impl DateFilter for WeekDayRange {
    fn filter(&self, date: NaiveDate) -> bool {
        match self {
            WeekDayRange::Fixed { range, nth, offset } => {
                // Apply the reverse of the offset
                let date = date - Duration::days(*offset);
                let range = (*range.start() as u8)..=(*range.end() as u8);
                let date_nth = (date.day() as u8 + 6) / 7;
                range.contains(&(date.weekday() as u8)) && nth.contains(&date_nth)
            }
            WeekDayRange::Holiday { .. } => todo!("Holiday not implemented yet"),
        }
    }
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

impl DateFilter for WeekRange {
    fn filter(&self, date: NaiveDate) -> bool {
        let week = date.iso_week().week() as u8;
        self.range.contains(&week) && (week - self.range.start()) % self.step == 0
    }
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

