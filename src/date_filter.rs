use std::cmp::Ordering;
use std::convert::TryInto;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate};

use opening_hours_syntax::rules::day::{self as ds, Month};

use crate::context::{Context, Localize};
use crate::opening_hours::DATE_LIMIT;
use crate::utils::dates::count_days_in_month;
use crate::utils::range::{RangeExt, WrappingRange};

/// Get the first valid date before give "yyyy/mm/dd", for example if
/// 2021/02/30 is given, this will return february 28th as 2021 is not a leap
/// year.
fn first_valid_ymd(year: i32, month: u32, day: u32) -> NaiveDate {
    (1..=day)
        .rev()
        .filter_map(|day| NaiveDate::from_ymd_opt(year, month, day))
        .next()
        .unwrap_or(DATE_LIMIT.date())
}

/// Generic trait to specify the behavior of a selector over dates.
pub trait DateFilter {
    fn filter<L>(&self, date: NaiveDate, ctx: &Context<L>) -> bool
    where
        L: Localize;

    /// Provide a lower bound to the next date with a different result to `filter`.
    fn next_change_hint<L>(&self, _date: NaiveDate, _ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize;
}

impl<T: DateFilter> DateFilter for [T] {
    fn filter<L>(&self, date: NaiveDate, ctx: &Context<L>) -> bool
    where
        L: Localize,
    {
        self.is_empty() || self.iter().any(|x| x.filter(date, ctx))
    }

    fn next_change_hint<L>(&self, date: NaiveDate, ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        self.iter()
            .map(|selector| selector.next_change_hint(date, ctx))
            .min()
            .unwrap_or_else(|| Some(DATE_LIMIT.date()))
    }
}

impl DateFilter for ds::DaySelector {
    fn filter<L>(&self, date: NaiveDate, ctx: &Context<L>) -> bool
    where
        L: Localize,
    {
        self.year.filter(date, ctx)
            && self.monthday.filter(date, ctx)
            && self.week.filter(date, ctx)
            && self.weekday.filter(date, ctx)
    }

    fn next_change_hint<L>(&self, date: NaiveDate, ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        // If there is no date filter, then all dates shall match
        if self.is_empty() {
            return date.succ_opt();
        }

        *[
            self.year.next_change_hint(date, ctx),
            self.monthday.next_change_hint(date, ctx),
            self.week.next_change_hint(date, ctx),
            self.weekday.next_change_hint(date, ctx),
        ]
        .iter()
        .min()
        .unwrap()
    }
}

impl DateFilter for ds::YearRange {
    fn filter<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> bool
    where
        L: Localize,
    {
        let Ok(year) = date.year().try_into() else {
            return false;
        };

        self.range.contains(&year) && (year - self.range.start()) % self.step == 0
    }

    fn next_change_hint<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        let Ok(curr_year) = date.year().try_into() else {
            return Some(DATE_LIMIT.date());
        };

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
                let round_up = |x: u16, d: u16| d * x.div_ceil(d); // get the first multiple of `d` greater than `x`.
                self.range.start() + round_up(curr_year - self.range.start(), self.step)
            }
        };

        Some(NaiveDate::from_ymd_opt(next_year.into(), 1, 1).unwrap_or(DATE_LIMIT.date()))
    }
}

impl DateFilter for ds::MonthdayRange {
    fn filter<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> bool
    where
        L: Localize,
    {
        let in_year = date.year() as u16;
        let in_month = Month::from_date(date);

        match self {
            ds::MonthdayRange::Month { year, range } => {
                year.unwrap_or(in_year) == in_year && range.wrapping_contains(&in_month)
            }
            ds::MonthdayRange::Date {
                start: (start, start_offset),
                end: (end, end_offset),
            } => {
                match (start, end) {
                    (
                        &ds::Date::Fixed { year: year_1, month: month_1, day: day_1 },
                        &ds::Date::Fixed { year: year_2, month: month_2, day: day_2 },
                    ) => {
                        let mut start = start_offset.apply(first_valid_ymd(
                            year_1.map(Into::into).unwrap_or(date.year()),
                            month_1.into(),
                            day_1.into(),
                        ));

                        // If no year is specified we can shift of as many years as needed.
                        if year_1.is_none() && start > date {
                            start = start_offset.apply(first_valid_ymd(
                                year_1.map(Into::into).unwrap_or(date.year()) - 1,
                                month_1.into(),
                                day_1.into(),
                            ))
                        }

                        let mut end = end_offset.apply(first_valid_ymd(
                            year_2.map(Into::into).unwrap_or(start.year()),
                            month_2.into(),
                            day_2.into(),
                        ));

                        // If no year is specified we can shift of as many years as needed.
                        if year_2.is_none() && end < start {
                            end = end_offset.apply(first_valid_ymd(
                                year_2.map(Into::into).unwrap_or(start.year()) + 1,
                                month_2.into(),
                                day_2.into(),
                            ))
                        }

                        (start..=end).contains(&date)
                    }
                    (_, ds::Date::Easter { year: _ }) | (ds::Date::Easter { year: _ }, _) => {
                        // TODO: Easter support
                        false
                    }
                }
            }
        }
    }

    fn next_change_hint<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        match self {
            ds::MonthdayRange::Month { range, year: None } => {
                let month = Month::from_date(date);

                if range.end().next() == *range.start() {
                    return Some(DATE_LIMIT.date());
                }

                let naive = {
                    if range.wrapping_contains(&month) {
                        NaiveDate::from_ymd_opt(date.year(), range.end().next() as _, 1)?
                    } else {
                        NaiveDate::from_ymd_opt(date.year(), range.start().next() as _, 1)?
                    }
                };

                if naive > date {
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
    fn filter<L>(&self, date: NaiveDate, ctx: &Context<L>) -> bool
    where
        L: Localize,
    {
        match self {
            ds::WeekDayRange::Fixed { range, offset, nth_from_start, nth_from_end } => {
                let date = date - Duration::days(*offset);
                let pos_from_start = (date.day() as u8 - 1) / 7;
                let pos_from_end = (count_days_in_month(date) - date.day() as u8) / 7;
                let range_u8 = (*range.start() as u8)..=(*range.end() as u8);

                range_u8.wrapping_contains(&(date.weekday() as u8))
                    && (nth_from_start[usize::from(pos_from_start)]
                        || nth_from_end[usize::from(pos_from_end)])
            }
            ds::WeekDayRange::Holiday { kind, offset } => {
                let calendar = match kind {
                    ds::HolidayKind::Public => &ctx.holidays.public,
                    ds::HolidayKind::School => &ctx.holidays.school,
                };

                let date = date - Duration::days(*offset);
                calendar.contains(date)
            }
        }
    }

    fn next_change_hint<L>(&self, date: NaiveDate, ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        match self {
            ds::WeekDayRange::Holiday { kind, offset } => Some({
                let calendar = match kind {
                    ds::HolidayKind::Public => &ctx.holidays.public,
                    ds::HolidayKind::School => &ctx.holidays.school,
                };

                let date_with_offset = date - Duration::days(*offset);

                if calendar.contains(date_with_offset) {
                    date.succ_opt()?
                } else {
                    calendar
                        .first_after(date_with_offset)
                        .map(|following| following + Duration::days(*offset))
                        .unwrap_or_else(|| DATE_LIMIT.date())
                }
            }),
            ds::WeekDayRange::Fixed {
                range: _,
                offset: _,
                nth_from_start: _,
                nth_from_end: _,
            } => None,
        }
    }
}

impl DateFilter for ds::WeekRange {
    fn filter<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> bool
    where
        L: Localize,
    {
        let week = date.iso_week().week() as u8;
        self.range.wrapping_contains(&week) && (week - self.range.start()) % self.step == 0
    }

    fn next_change_hint<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        let week = date.iso_week().week() as u8;

        if self.range.wrapping_contains(&week) {
            let end_week = {
                if self.step == 1 {
                    *self.range.end() % 53 + 1
                } else if (week - self.range.start()) % self.step == 0 {
                    (date.iso_week().week() as u8 % 53) + 1
                } else {
                    return None;
                }
            };

            let end_year = {
                if date.iso_week().week() <= u32::from(end_week) {
                    date.iso_week().year()
                } else {
                    date.iso_week().year() + 1
                }
            };

            Some(
                NaiveDate::from_isoywd_opt(end_year, end_week.into(), ds::Weekday::Mon)
                    .unwrap_or(DATE_LIMIT.date()),
            )
        } else if week < *self.range.start() {
            Some(
                NaiveDate::from_isoywd_opt(
                    date.iso_week().year(),
                    (*self.range.start()).into(),
                    ds::Weekday::Mon,
                )
                .unwrap_or(DATE_LIMIT.date()),
            )
        } else {
            Some(
                NaiveDate::from_isoywd_opt(
                    date.year() + 1,
                    (*self.range.start()).into(),
                    ds::Weekday::Mon,
                )
                .unwrap_or(DATE_LIMIT.date()),
            )
        }
    }
}
