use std::convert::TryInto;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate};

use crate::opening_hours::DATE_LIMIT;
use crate::utils::range::wrapping_range_contains;
use opening_hours_syntax::rules::day as ds;
use opening_hours_syntax::sorted_vec::UniqueSortedVec;

/// Generic trait to specify the behavior of a selector over dates.
pub trait DateFilter {
    fn filter(&self, date: NaiveDate, holidays: &UniqueSortedVec<NaiveDate>) -> bool;

    /// Provide a lower bound to the next date with a different result to `filter`.
    fn next_change_hint(
        &self,
        _date: NaiveDate,
        _holidays: &UniqueSortedVec<NaiveDate>,
    ) -> Option<NaiveDate> {
        None
    }
}

impl<T: DateFilter> DateFilter for [T] {
    fn filter(&self, date: NaiveDate, holidays: &UniqueSortedVec<NaiveDate>) -> bool {
        self.is_empty() || self.iter().any(|x| x.filter(date, holidays))
    }

    fn next_change_hint(
        &self,
        date: NaiveDate,
        holidays: &UniqueSortedVec<NaiveDate>,
    ) -> Option<NaiveDate> {
        self.iter()
            .map(|selector| selector.next_change_hint(date, holidays))
            .min()
            .unwrap_or_else(|| Some(DATE_LIMIT.date()))
    }
}

impl DateFilter for ds::DaySelector {
    fn filter(&self, date: NaiveDate, holidays: &UniqueSortedVec<NaiveDate>) -> bool {
        self.year.filter(date, holidays)
            && self.monthday.filter(date, holidays)
            && self.week.filter(date, holidays)
            && self.weekday.filter(date, holidays)
    }

    fn next_change_hint(
        &self,
        date: NaiveDate,
        holidays: &UniqueSortedVec<NaiveDate>,
    ) -> Option<NaiveDate> {
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
    fn filter(&self, date: NaiveDate, _holidays: &UniqueSortedVec<NaiveDate>) -> bool {
        let year: u16 = date.year().try_into().unwrap();
        self.range.contains(&year) && (year - self.range.start()) % self.step == 0
    }

    fn next_change_hint(
        &self,
        date: NaiveDate,
        _holidays: &UniqueSortedVec<NaiveDate>,
    ) -> Option<NaiveDate> {
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

impl DateFilter for ds::MonthdayRange {
    fn filter(&self, date: NaiveDate, _holidays: &UniqueSortedVec<NaiveDate>) -> bool {
        let in_year = date.year() as u16;
        let in_month = ds::Month::from_u8(date.month() as u8).expect("invalid month value");

        match self {
            ds::MonthdayRange::Month { year, range } => {
                year.unwrap_or(in_year) == in_year && wrapping_range_contains(range, &in_month)
            }
            ds::MonthdayRange::Date {
                start: (start, start_offset),
                end: (end, end_offset),
            } => match (&start, end) {
                (
                    ds::Date::Fixed {
                        year: year_1,
                        month: month_1,
                        day: day_1,
                    },
                    ds::Date::Fixed {
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

impl DateFilter for ds::WeekDayRange {
    fn filter(&self, date: NaiveDate, holidays: &UniqueSortedVec<NaiveDate>) -> bool {
        match self {
            ds::WeekDayRange::Fixed { range, nth, offset } => {
                let date = date - Duration::days(*offset);
                let date_nth = (date.day() as u8 - 1) / 7;
                let range_u8 = (*range.start() as u8)..=(*range.end() as u8);
                wrapping_range_contains(&range_u8, &(date.weekday() as u8))
                    && nth[usize::from(date_nth)]
            }
            ds::WeekDayRange::Holiday { kind, offset } => match kind {
                ds::HolidayKind::Public => {
                    let date = date - Duration::days(*offset);
                    holidays.contains(&date)
                }
                ds::HolidayKind::School => {
                    eprintln!("[WARN] school holidays are not supported, thus ignored");
                    false
                }
            },
        }
    }

    fn next_change_hint(
        &self,
        date: NaiveDate,
        holidays: &UniqueSortedVec<NaiveDate>,
    ) -> Option<NaiveDate> {
        match self {
            ds::WeekDayRange::Holiday {
                kind: ds::HolidayKind::Public,
                offset,
            } => Some({
                let date_with_offset = date - Duration::days(*offset);

                if holidays.contains(&date_with_offset) {
                    date.succ_opt()?
                } else {
                    holidays
                        .find_first_following(&date_with_offset)
                        .map(|following| *following + Duration::days(*offset))
                        .unwrap_or_else(|| DATE_LIMIT.date())
                }
            }),
            _ => None,
        }
    }
}

impl DateFilter for ds::WeekRange {
    fn filter(&self, date: NaiveDate, _holidays: &UniqueSortedVec<NaiveDate>) -> bool {
        let week = date.iso_week().week() as u8;
        wrapping_range_contains(&self.range, &week) && (week - self.range.start()) % self.step == 0
    }
}
