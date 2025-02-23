use std::cmp::Ordering;
use std::convert::TryInto;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate, Weekday};

use opening_hours_syntax::rules::day::{self as ds, Month};

use crate::Context;
use crate::localization::Localize;
use crate::opening_hours::DATE_END;
use crate::utils::dates::{count_days_in_month, easter};
use crate::utils::range::{RangeExt, WrappingRange};

/// Get the first valid date before given "yyyy/mm/dd", for example if
/// 2021/02/30 is given, this will return february 28th as 2021 is not a leap
/// year.
fn first_valid_ymd(year: i32, month: u32, day: u32) -> NaiveDate {
    (1..=day)
        .rev()
        .filter_map(|day| NaiveDate::from_ymd_opt(year, month, day))
        .next()
        .unwrap_or(DATE_END.date())
}

/// Find next change from iterators of "starting of an interval" to "end of an
/// interval".
fn next_change_from_bounds(
    date: NaiveDate,
    bounds_start: impl IntoIterator<Item = NaiveDate>,
    bounds_end: impl IntoIterator<Item = NaiveDate>,
) -> Option<NaiveDate> {
    let mut bounds_start = bounds_start.into_iter().peekable();
    let mut bounds_end = bounds_end.into_iter().peekable();

    loop {
        match (bounds_start.peek().copied(), bounds_end.peek().copied()) {
            // The date is after the end of the last interval
            (None, None) => return None,
            (None, Some(end)) => {
                if end >= date {
                    // The date belongs to the last interval
                    return end.succ_opt();
                } else {
                    // The date is after the last interval end
                    return None;
                }
            }
            (Some(start), None) => {
                if start > date {
                    // The date is before the first interval
                    return Some(start);
                } else {
                    // The date belongs to the last interval, which never ends.
                    return None;
                }
            }
            (Some(start), Some(end)) => {
                if start <= end {
                    if (start..=end).contains(&date) {
                        // We found an interval the date belongs to
                        return end.succ_opt();
                    }

                    bounds_start.next();
                } else {
                    if (end.succ_opt()?..start).contains(&date) {
                        // We found an inbetween of intervals the date belongs to
                        return Some(start);
                    }

                    bounds_end.next();
                }
            }
        }
    }
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
            .unwrap_or_else(|| Some(DATE_END.date()))
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
        let range = **self.range.start()..=**self.range.end();

        let Ok(year) = date.year().try_into() else {
            return false;
        };

        range.wrapping_contains(&year)
            && (year.checked_sub(*range.start()))
                .or_else(|| range.start().checked_sub(year))
                .unwrap_or(0)
                % self.step
                == 0
    }

    fn next_change_hint<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        let range = **self.range.start()..=**self.range.end();

        let Ok(curr_year) = date.year().try_into() else {
            return Some(DATE_END.date());
        };

        if self.range.start() > self.range.end() {
            return None; // TODO
        }

        let next_year = {
            if *range.end() < curr_year {
                // 1. time exceeded the range, the state won't ever change
                return Some(DATE_END.date());
            } else if curr_year < *range.start() {
                // 2. time didn't reach the range yet
                *range.start()
            } else if self.step == 1 {
                // 3. time is in the range and step is naive
                *range.end() + 1
            } else if (curr_year - range.start()) % self.step == 0 {
                // 4. time matches the range with step >= 2
                curr_year + 1
            } else {
                // 5. time is in the range but doesn't match the step
                let round_up = |x: u16, d: u16| d * x.div_ceil(d); // get the first multiple of `d` greater than `x`.
                range.start() + round_up(curr_year - range.start(), self.step)
            }
        };

        Some(NaiveDate::from_ymd_opt(next_year.into(), 1, 1).unwrap_or(DATE_END.date()))
    }
}

/// Project date on a given year.
fn date_on_year(date: ds::Date, for_year: i32) -> Option<NaiveDate> {
    match date {
        ds::Date::Easter { year } => easter(year.map(Into::into).unwrap_or(for_year)),
        ds::Date::Fixed { year, month, day } => Some(first_valid_ymd(
            year.map(Into::into).unwrap_or(for_year),
            month.into(),
            day.into(),
        )),
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
                let mut start_date = match date_on_year(*start, date.year()) {
                    Some(date) => start_offset.apply(date),
                    None => return false,
                };

                if start_date > date {
                    start_date = match date_on_year(*start, date.year() - 1) {
                        Some(date) => start_offset.apply(date),
                        None => return false,
                    };
                }

                let mut end_date = match date_on_year(*end, start_date.year()) {
                    Some(date) => end_offset.apply(date),
                    None => return false,
                };

                if end_date < start_date {
                    end_date = match date_on_year(*end, start_date.year() + 1) {
                        Some(date) => end_offset.apply(date),
                        None => return false,
                    };
                }

                (start_date..=end_date).contains(&date)
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
                    return Some(DATE_END.date());
                }

                let naive = {
                    if range.wrapping_contains(&month) {
                        NaiveDate::from_ymd_opt(date.year(), range.end().next() as _, 1)?
                    } else {
                        NaiveDate::from_ymd_opt(date.year(), *range.start() as _, 1)?
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
                    Ordering::Greater => DATE_END.date(),
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
                    Ordering::Greater => DATE_END.date(),
                })
            }
            ds::MonthdayRange::Date {
                start: (start, start_offset),
                end: (end, end_offset),
            } => {
                let year = date.year();

                next_change_from_bounds(
                    date,
                    (year - 1..=year + 1)
                        .filter_map(|y| date_on_year(*start, y))
                        .map(|d| start_offset.apply(d)),
                    (year - 1..=year + 1)
                        .filter_map(|y| date_on_year(*end, y))
                        .map(|d| end_offset.apply(d)),
                )
            }
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
                if *range.start() as u8 > *range.end() as u8 {
                    // Handle wrapping ranges
                    return ds::WeekDayRange::Fixed {
                        range: *range.start()..=Weekday::Sun,
                        offset: *offset,
                        nth_from_start: *nth_from_start,
                        nth_from_end: *nth_from_end,
                    }
                    .filter(date, ctx)
                        || ds::WeekDayRange::Fixed {
                            range: Weekday::Mon..=*range.end(),
                            offset: *offset,
                            nth_from_start: *nth_from_start,
                            nth_from_end: *nth_from_end,
                        }
                        .filter(date, ctx);
                }

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
                        .unwrap_or_else(|| DATE_END.date())
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
        let range = **self.range.start()..=**self.range.end();

        range.wrapping_contains(&week)
            // TODO: what happens when week < range.start ?
            && week.saturating_sub(*range.start()) % self.step == 0
    }

    fn next_change_hint<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        let week = date.iso_week().week() as u8;
        let range = **self.range.start()..=**self.range.end();

        if self.range.start() > self.range.end() {
            // TODO: wrapping implemented well?
            return None;
        }

        let weeknum = u32::from({
            if range.wrapping_contains(&week) {
                if self.step == 1 {
                    *range.end() % 54 + 1
                } else if (week - range.start()) % self.step == 0 {
                    (date.iso_week().week() as u8 % 54) + 1
                } else {
                    return None;
                }
            } else {
                *range.start()
            }
        });

        let mut res =
            NaiveDate::from_isoywd_opt(date.iso_week().year(), weeknum, ds::Weekday::Mon)?;

        while res <= date {
            res = NaiveDate::from_isoywd_opt(res.iso_week().year() + 1, weeknum, ds::Weekday::Mon)?;
        }

        Some(res)
    }
}
