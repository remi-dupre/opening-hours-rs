use std::cmp::Ordering;
use std::convert::TryInto;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate};

use compact_calendar::CompactCalendar;
use opening_hours_syntax::rules::day as ds;

use crate::opening_hours::DATE_LIMIT;
use crate::utils::range::{RangeExt, WrappingRange};

/// Generic trait to specify the behavior of a selector over dates.
pub trait DateFilter {
    fn filter(&self, date: NaiveDate, holidays: &CompactCalendar) -> bool;

    /// Provide a lower bound to the next date with a different result to `filter`.
    fn next_change_hint(&self, _date: NaiveDate, _holidays: &CompactCalendar) -> Option<NaiveDate> {
        None
    }
}

impl<T: DateFilter> DateFilter for [T] {
    fn filter(&self, date: NaiveDate, holidays: &CompactCalendar) -> bool {
        self.is_empty() || self.iter().any(|x| x.filter(date, holidays))
    }

    fn next_change_hint(&self, date: NaiveDate, holidays: &CompactCalendar) -> Option<NaiveDate> {
        self.iter()
            .map(|selector| selector.next_change_hint(date, holidays))
            .min()
            .unwrap_or_else(|| Some(DATE_LIMIT.date()))
    }
}

impl DateFilter for ds::DaySelector {
    fn filter(&self, date: NaiveDate, holidays: &CompactCalendar) -> bool {
        self.year.filter(date, holidays)
            && self.monthday.filter(date, holidays)
            && self.week.filter(date, holidays)
            && self.weekday.filter(date, holidays)
    }

    fn next_change_hint(&self, date: NaiveDate, holidays: &CompactCalendar) -> Option<NaiveDate> {
        *[
            self.year.next_change_hint(date, holidays),
            self.monthday.next_change_hint(date, holidays),
            self.week.next_change_hint(date, holidays),
            self.weekday.next_change_hint(date, holidays),
        ]
        .iter()
        .min()
        .unwrap()
    }
}

impl DateFilter for ds::YearRange {
    fn filter(&self, date: NaiveDate, _holidays: &CompactCalendar) -> bool {
        let year: u16 = date.year().try_into().unwrap();
        self.range.contains(&year) && (year - self.range.start()) % self.step == 0
    }

    fn next_change_hint(&self, date: NaiveDate, _holidays: &CompactCalendar) -> Option<NaiveDate> {
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

        Some(
            NaiveDate::from_ymd_opt(next_year.into(), 1, 1)
                .expect("invalid year range: end bound is too large"),
        )
    }
}

impl DateFilter for ds::MonthdayRange {
    fn filter(&self, date: NaiveDate, _holidays: &CompactCalendar) -> bool {
        let in_year = date.year() as u16;
        let in_month = date.month().try_into().expect("invalid month value");

        match self {
            ds::MonthdayRange::Month { year, range } => {
                year.unwrap_or(in_year) == in_year && range.wrapping_contains(&in_month)
            }
            ds::MonthdayRange::Date {
                start: (start, start_offset),
                end: (end, end_offset),
            } => {
                match (&start, end) {
                    (
                        ds::Date::Fixed { year: year_1, month: month_1, day: day_1 },
                        ds::Date::Fixed { year: year_2, month: month_2, day: day_2 },
                    ) => {
                        let start = NaiveDate::from_ymd_opt(
                            year_1.map(|x| x as i32).unwrap_or_else(|| date.year()),
                            *month_1 as u32,
                            *day_1 as u32,
                        )
                        .expect("invalid date range: start bound is too large");

                        let mut start = start_offset.apply(start);

                        // If no year is specified we can shift of as many years as needed.
                        if year_1.is_none() {
                            start = start.with_year(date.year()).unwrap();

                            if start > date {
                                start = start.with_year(start.year() - 1).expect("year overflow");
                            }
                        }

                        let end = NaiveDate::from_ymd_opt(
                            year_2.map(|x| x as i32).unwrap_or_else(|| start.year()),
                            *month_2 as u32,
                            *day_2 as u32,
                        )
                        .expect("invalid date range: end bound is too large");

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
                }
            }
        }
    }

    fn next_change_hint(&self, date: NaiveDate, _holidays: &CompactCalendar) -> Option<NaiveDate> {
        match self {
            ds::MonthdayRange::Month { range, year: None } => {
                let month = date.month().try_into().expect("invalid month value");

                let naive = {
                    if range.wrapping_contains(&month) {
                        NaiveDate::from_ymd_opt(date.year(), range.end().next() as _, 1)?
                    } else {
                        NaiveDate::from_ymd_opt(date.year(), range.start().next() as _, 1)?
                    }
                };

                if naive >= date {
                    Some(naive)
                } else {
                    naive.with_year(naive.year() + 1)
                }
            }
            ds::MonthdayRange::Month { range, year: Some(year) } => {
                let year: i32 = (*year).into();
                let start_month: u32 = *range.start() as _;
                let end_month: u32 = *range.end() as _;

                let start = NaiveDate::from_ymd_opt(year, *range.start() as _, 1)?;
                let end = {
                    if start_month <= end_month && end_month < 12 {
                        NaiveDate::from_ymd_opt(year, end_month + 1, 1)?
                    } else {
                        NaiveDate::from_ymd_opt(year + 1, end_month % 12 + 1, 1)?
                    }
                };

                Some(match (start..end).compare(&date) {
                    Ordering::Less => start,
                    Ordering::Equal => end,
                    Ordering::Greater => DATE_LIMIT.date(),
                })
            }
            ds::MonthdayRange::Date {
                start:
                    (
                        ds::Date::Fixed {
                            year: Some(start_year),
                            month: start_month,
                            day: start_day,
                        },
                        start_offset,
                    ),
                end:
                    (ds::Date::Fixed { year: end_year, month: end_month, day: end_day }, end_offset),
            } => {
                let start = start_offset.apply(NaiveDate::from_ymd_opt(
                    (*start_year).into(),
                    *start_month as _,
                    (*start_day).into(),
                )?);

                let end = {
                    let candidate = end_offset.apply(NaiveDate::from_ymd_opt(
                        end_year.unwrap_or_else(|| *start_year).into(),
                        *end_month as _,
                        (*end_day).into(),
                    )?);

                    if start <= candidate {
                        candidate
                    } else {
                        candidate.with_year(candidate.year() + 1)?
                    }
                };

                Some(match (start..end).compare(&date) {
                    Ordering::Less => start,
                    Ordering::Equal => end + Duration::days(1),
                    Ordering::Greater => DATE_LIMIT.date(),
                })
            }
            ds::MonthdayRange::Date {
                start:
                    (ds::Date::Fixed { year: None, month: start_month, day: start_day }, start_offset),
                end: (ds::Date::Fixed { year: None, month: end_month, day: end_day }, end_offset),
            } => {
                let end = {
                    let mut candidate = end_offset.apply(NaiveDate::from_ymd_opt(
                        date.year(),
                        *end_month as _,
                        (*end_day).into(),
                    )?);

                    while candidate < date {
                        candidate = candidate.with_year(candidate.year() + 1)?;
                    }

                    candidate
                };

                let start = {
                    let candidate = start_offset.apply(NaiveDate::from_ymd_opt(
                        end.year(),
                        *start_month as _,
                        (*start_day).into(),
                    )?);

                    if candidate > end {
                        candidate.with_year(end.year() - 1)?
                    } else {
                        candidate
                    }
                };

                // We already enforced end >= date, thus we only need to compare it to the start.
                Some({
                    if start <= date {
                        // date is in [start, end]
                        end.succ_opt()?
                    } else {
                        // date is before [start, end]
                        start
                    }
                })
            }
            _ => None,
        }
    }
}

impl DateFilter for ds::WeekDayRange {
    fn filter(&self, date: NaiveDate, holidays: &CompactCalendar) -> bool {
        match self {
            ds::WeekDayRange::Fixed { range, nth, offset } => {
                let date = date - Duration::days(*offset);
                let date_nth = (date.day() as u8 - 1) / 7;
                let range_u8 = (*range.start() as u8)..=(*range.end() as u8);
                range_u8.wrapping_contains(&(date.weekday() as u8)) && nth[usize::from(date_nth)]
            }
            ds::WeekDayRange::Holiday { kind, offset } => match kind {
                ds::HolidayKind::Public => {
                    let date = date - Duration::days(*offset);
                    holidays.contains(date)
                }
                ds::HolidayKind::School => {
                    eprintln!("[WARN] school holidays are not supported, thus ignored");
                    false
                }
            },
        }
    }

    fn next_change_hint(&self, date: NaiveDate, holidays: &CompactCalendar) -> Option<NaiveDate> {
        match self {
            ds::WeekDayRange::Holiday { kind: ds::HolidayKind::Public, offset } => Some({
                let date_with_offset = date - Duration::days(*offset);

                if holidays.contains(date_with_offset) {
                    date.succ_opt()?
                } else {
                    holidays
                        .first_after(date_with_offset)
                        .map(|following| following + Duration::days(*offset))
                        .unwrap_or_else(|| DATE_LIMIT.date())
                }
            }),
            _ => None,
        }
    }
}

impl DateFilter for ds::WeekRange {
    fn filter(&self, date: NaiveDate, _holidays: &CompactCalendar) -> bool {
        let week = date.iso_week().week() as u8;
        self.range.wrapping_contains(&week) && (week - self.range.start()) % self.step == 0
    }
}
