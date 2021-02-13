use std::collections::{BTreeSet, HashMap};
use std::convert::TryInto;
use std::env;
use std::ops::RangeInclusive;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate};
use once_cell::sync::Lazy;

use crate::time_domain::DATE_LIMIT;
use crate::utils::wrapping_range_contains;

// Load dates from file

macro_rules! load_dates {
    ( $( $region: literal ),+ ) => {{
        let mut regions = HashMap::new();

        $(
            let dates = include_bytes!(concat!(env!("HOLIDAYS_DIR"), "/", $region, ".bin"))
                .chunks_exact(4)
                .map(|bytes| {
                    let days = i32::from_le_bytes(bytes.try_into().unwrap());
                    NaiveDate::from_num_days_from_ce(days)
                })
                .collect();

            regions.insert($region, dates);
        )+

        regions
    }};
}

pub static REGION_HOLIDAYS: Lazy<HashMap<&str, BTreeSet<NaiveDate>>> = Lazy::new(|| {
    load_dates!(
        "CH-VS", "CA-YT", "BR-CE", "US-LA", "BR-MA", "PL", "CA-BC", "BR-PA", "DE", "CA-NT",
        "BR-RJ", "BR", "IE", "US-MA", "CH-NW", "CH-AI", "US-MD", "CN", "US-RI", "CH-ZG", "GR",
        "GB-NIR", "CH-SZ", "CH-FR", "AR", "DE-NI", "BB", "US-KY", "NL", "US-WV", "DE-BW", "BR-MG",
        "US-OK", "BR-DF", "CA-NL", "US-IL", "AU-ACT", "ST", "US-UT", "HR", "AU-NT", "TR", "US-FL",
        "HU", "AU-WA", "DE-ST", "US-AK", "EE", "HK", "CH-GR", "BJ", "BR-AL", "US-OR", "MX", "PY",
        "US-GU", "SE", "FI", "CL", "BR-MT", "CH-BS", "US-VA", "RO", "ES-VC", "US-CO", "TW",
        "US-NC", "US-AZ", "US-NE", "NZ", "BR-AC", "BR-RN", "FR", "ES-PV", "ES-EX", "US-WI",
        "BR-RR", "RU", "CA-PE", "SG", "AU-VIC", "GB", "US-DE", "IS", "CH-ZH", "CH-AG", "CA-AB",
        "DE-TH", "CA-SK", "US-CA", "BR-SC", "DE-RP", "ES-CM", "US-NV", "DE-BB", "CH-GL", "DE-BE",
        "US-WA", "MY", "CA-ON", "CA-NB", "US-CT", "BR-SE", "BR-SP", "LT", "LV", "MC", "CO",
        "US-IA", "DE-HE", "ES-GA", "CH-NE", "CH-VD", "CH-UR", "US-TN", "CI", "US-ME", "ES-CN",
        "DE-HB", "US-MN", "BR-RS", "CA-QC", "KR", "US-IN", "US-TX", "CH-JU", "JP", "CZ", "UA",
        "AU-QLD", "KY", "AU", "US-NY", "DZ", "US-MS", "CY", "DE-HH", "CH-SO", "CH-GE", "DE-MV",
        "AU-SA", "US-SC", "BR-PE", "DE-SH", "US-MI", "CA-MB", "US-ND", "CH-TI", "US-OH", "BR-RO",
        "DE-NW", "US-VT", "MH", "BR-PR", "NO", "US-NM", "DE-SN", "US-PA", "US-HI", "ZA", "ES-MD",
        "US-SD", "BR-AP", "LU", "US-MT", "DE-BY", "KE", "DK", "IL", "ES-RI", "ES-CB", "CA-NU",
        "BR-PI", "PA", "US-MO", "CA-NS", "CH", "US", "DE-SL", "AO", "US-AR", "QA", "BR-GO", "BE",
        "BR-BA", "US-NH", "SI", "US-AS", "ES-NA", "BR-MS", "BR-TO", "ES-IB", "ES-AN", "US-GA",
        "CH-BL", "BY", "ES", "US-WY", "CH-TG", "BR-AM", "MZ", "US-ID", "ES-CT", "PT", "MG",
        "ES-AR", "AT", "CH-BE", "ES-AS", "US-DC", "CH-AR", "ES-MC", "US-KS", "CA", "AU-NSW",
        "US-AL", "CH-OW", "MT", "RS", "SK", "BG", "CH-SG", "BR-ES", "US-NJ", "CH-LU", "IT",
        "ES-CL", "CH-SH", "BR-PB"
    )
});

// DateFilter

pub type Weekday = chrono::Weekday;

/// Generic trait to specify the behavior of a selector over dates.
pub trait DateFilter {
    fn filter(&self, date: NaiveDate, region: Option<&str>) -> bool;

    /// Provide a lower bound to the next date with a different result to `filter`.
    fn next_change_hint(&self, _date: NaiveDate) -> Option<NaiveDate> {
        None
    }
}

impl<T: DateFilter> DateFilter for [T] {
    fn filter(&self, date: NaiveDate, region: Option<&str>) -> bool {
        self.is_empty() || self.iter().any(|x| x.filter(date, region))
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
    fn filter(&self, date: NaiveDate, region: Option<&str>) -> bool {
        self.year.filter(date, region)
            && self.monthday.filter(date, region)
            && self.week.filter(date, region)
            && self.weekday.filter(date, region)
    }

    fn next_change_hint(&self, date: NaiveDate) -> Option<NaiveDate> {
        if self.monthday.is_empty() && self.week.is_empty() && self.weekday.is_empty() {
            self.year
                .iter()
                .map(|year_selector| year_selector.next_change_hint(date))
                .min()
                .unwrap_or_else(|| Some(DATE_LIMIT.date()))
        } else {
            None
        }
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
    fn filter(&self, date: NaiveDate, _region: Option<&str>) -> bool {
        let year: u16 = date.year().try_into().unwrap();
        self.range.contains(&year) && (year - self.range.start()) % self.step == 0
    }

    fn next_change_hint(&self, date: NaiveDate) -> Option<NaiveDate> {
        let curr_year: u16 = date.year().try_into().unwrap();

        let next_year = {
            if *self.range.end() < curr_year {
                // 1. time exceeded the range, the state won't ever change
                return Some(DATE_LIMIT.date());
            } else if curr_year < *self.range.start() {
                // 2. time didn't reach the range yet
                *self.range.start()
            } else if self.step == 1 {
                // 3. time is in the range and step is naive
                *self.range.end() + 1
            } else if (curr_year - self.range.start()) % self.step == 0 {
                // 4. time matches the range with step >= 2
                curr_year + 1
            } else {
                // 5. time is in the range but doesn't match the step
                let round_up = |x: u16, d: u16| d * ((x + d - 1) / d); // get the first multiple of `d` greater than `x`.
                self.range.start() + round_up(curr_year - self.range.start(), self.step)
            }
        };

        Some(NaiveDate::from_ymd(next_year.into(), 1, 1))
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
    fn filter(&self, date: NaiveDate, _region: Option<&str>) -> bool {
        let in_year = date.year() as u16;
        let in_month = Month::from_u8(date.month() as u8).expect("invalid month value");

        match self {
            MonthdayRange::Month { year, range } => {
                year.unwrap_or(in_year) == in_year && wrapping_range_contains(range, &in_month)
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
                        year_1.map(|x| x as i32).unwrap_or_else(|| date.year()),
                        *month_1 as u32,
                        *day_1 as u32,
                    );

                    let mut start = start_offset.apply(start);

                    // If no year is specified we can shift of as many years as needed.
                    if year_1.is_none() {
                        start = start.with_year(date.year()).unwrap();

                        if start > date {
                            start = start.with_year(start.year() - 1).expect("year overflow");
                        }
                    }

                    let end = NaiveDate::from_ymd(
                        year_2.map(|x| x as i32).unwrap_or_else(|| start.year()),
                        *month_2 as u32,
                        *day_2 as u32,
                    );

                    let mut end = end_offset.apply(end);

                    // If no year is specified we can shift of as many years as needed.
                    if year_2.is_none() {
                        end = end.with_year(start.year()).unwrap();

                        // If end's month is prior that start's month, end must be next year.
                        if end < start {
                            end = end.with_year(end.year() + 1).expect("year overflow")
                        }
                    }

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
        offset: i64,
        nth: [bool; 5],
    },
    Holiday {
        kind: HolidayKind,
        offset: i64,
    },
}

impl DateFilter for WeekDayRange {
    fn filter(&self, date: NaiveDate, region: Option<&str>) -> bool {
        match self {
            WeekDayRange::Fixed { range, nth, offset } => {
                let date = date - Duration::days(*offset);
                let date_nth = (date.day() as u8 - 1) / 7;
                let range_u8 = (*range.start() as u8)..=(*range.end() as u8);
                wrapping_range_contains(&range_u8, &(date.weekday() as u8))
                    && nth[usize::from(date_nth)]
            }
            WeekDayRange::Holiday { kind, offset } => match kind {
                HolidayKind::Public => {
                    if let Some(region) = region {
                        let date = date - Duration::days(*offset);
                        REGION_HOLIDAYS[region].contains(&date)
                    } else {
                        false
                    }
                }
                HolidayKind::School => {
                    eprintln!("[WARN] school holidays are ignored");
                    false
                }
            },
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
    fn filter(&self, date: NaiveDate, _region: Option<&str>) -> bool {
        let week = date.iso_week().week() as u8;
        wrapping_range_contains(&self.range, &week) && (week - self.range.start()) % self.step == 0
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

#[derive(Debug)]
pub struct InvalidMonth;

impl Month {
    pub fn from_u8(x: u8) -> Result<Self, InvalidMonth> {
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
            _ => return Err(InvalidMonth),
        })
    }

    pub fn next(self) -> Self {
        let num = self as u8;
        Self::from_u8((num % 12) + 1).unwrap()
    }
}
