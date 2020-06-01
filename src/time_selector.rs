use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate};

use crate::time_domain as td;

/// ---
/// --- DateSelector
/// ---

pub trait DateFilter {
    fn filter(&self, date: &NaiveDate) -> bool;
}

impl DateFilter for td::YearRange {
    fn filter(&self, date: &NaiveDate) -> bool {
        let year = date.year() as u16;
        self.range.contains(&year) && (year - self.range.start()) % self.step == 0
    }
}

impl DateFilter for td::MonthdayRange {
    fn filter(&self, date: &NaiveDate) -> bool {
        let in_year = date.year() as u16;
        let in_month = td::Month::from_u8(date.month() as u8).expect("inalid month value");

        match self {
            td::MonthdayRange::Month { year, range } => {
                year.unwrap_or(in_year) == in_year && range.contains(&in_month)
            }
            td::MonthdayRange::Date {
                start: (start, start_offset),
                end: (end, end_offset),
            } => match (&start, end) {
                (
                    td::Date::Fixed {
                        year: year_1,
                        month: month_1,
                        day: day_1,
                    },
                    td::Date::Fixed {
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

                    let start = start_offset.apply(&start);
                    let end = end_offset.apply(&end);
                    (start..=end).contains(&date)
                }
                _ => todo!("Easter not implemented yet"),
            },
        }
    }
}

impl DateFilter for td::WeekRange {
    fn filter(&self, date: &NaiveDate) -> bool {
        let week = date.iso_week().week() as u8;
        self.range.contains(&week) && (week - self.range.start()) % self.step == 0
    }
}

impl DateFilter for td::WeekDayRange {
    fn filter(&self, date: &NaiveDate) -> bool {
        match self {
            td::WeekDayRange::Fixed { range, nth, offset } => {
                // Apply the reverse of the offset
                let date = *date - Duration::days(*offset);
                let range = (*range.start() as u8)..=(*range.end() as u8);
                let date_nth = (date.day() as u8 + 6) / 7;
                range.contains(&(date.weekday() as u8)) && nth.contains(&date_nth)
            }
            td::WeekDayRange::Holiday { .. } => todo!("Holiday not implemented yet"),
        }
    }
}
