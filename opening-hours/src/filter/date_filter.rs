use std::ops::RangeInclusive;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate, Weekday};

use opening_hours_syntax::rules::day::{self as ds, Date, Month, WeekNum, Year};

use crate::localization::Localize;
use crate::opening_hours::{DATE_END, DATE_START};
use crate::utils::dates::{count_days_in_month, easter};
use crate::Context;

/// Get the first valid date before given "yyyy/mm/dd", for example if 2021/02/30 is given, this
/// will return february 28th as 2021 is not a leap year.
fn valid_ymd_before(year: Year, month: u32, day: u32) -> NaiveDate {
    debug_assert!((1..=31).contains(&day));

    NaiveDate::from_ymd_opt(*year, month, day)
        .into_iter()
        .chain(
            (28..day)
                .rev()
                .filter_map(|day| NaiveDate::from_ymd_opt(*year, month, day)),
        )
        .next()
        .unwrap_or(DATE_END.date())
}

/// Get the first valid date after given "yyyy/mm/dd", for example if 2021/02/30 is given, this
/// will return march 1st of 2021.
fn valid_ymd_after(year: Year, month: u32, day: u32) -> NaiveDate {
    debug_assert!((1..=31).contains(&day));

    NaiveDate::from_ymd_opt(*year, month, day)
        .into_iter()
        .chain(
            (28..day)
                .rev()
                .filter_map(|day| NaiveDate::from_ymd_opt(*year, month, day)?.succ_opt()),
        )
        .next()
        .unwrap_or(DATE_END.date())
}

fn ensure_increasing_iter<T: Ord>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
    let mut iter = iter.peekable();

    std::iter::from_fn(move || {
        let val = iter.next()?;
        while iter.next_if(|next_val| *next_val <= val).is_some() {}
        Some(val)
    })
}

fn intervals_from_bounds(
    bounds_start: impl IntoIterator<Item = NaiveDate>,
    bounds_end: impl IntoIterator<Item = NaiveDate>,
) -> impl Iterator<Item = RangeInclusive<NaiveDate>> {
    let mut bounds_start = ensure_increasing_iter(bounds_start.into_iter()).peekable();
    let mut bounds_end = ensure_increasing_iter(bounds_end.into_iter()).peekable();
    let start_is_empty = bounds_start.peek().is_none();

    std::iter::from_fn(move || {
        if let Some(start) = bounds_start.peek() {
            while bounds_end.next_if(|end| end < start).is_some() {}
        }

        let range = match (bounds_start.peek().copied(), bounds_end.peek().copied()) {
            // The date is after the end of the last interval
            (None, None) => return None,
            (None, Some(end)) => {
                if start_is_empty {
                    (&mut bounds_end).for_each(|_| {});
                    DATE_START.date()..=end
                } else {
                    return None;
                }
            }
            (Some(start), None) => {
                bounds_start.next();
                start..=DATE_END.date()
            }
            (Some(start), Some(end)) if (start <= end) => {
                if start == end {
                    bounds_end.next();
                }

                bounds_start.next();
                start..=end
            }
            (Some(_), Some(_)) => {
                unreachable!()
            }
        };

        Some(range)
    })
}

fn is_open_from_intervals(
    date: NaiveDate,
    mut intervals: impl Iterator<Item = RangeInclusive<NaiveDate>>,
) -> bool {
    let Some(first_interval) = intervals.find(|rg| *rg.end() >= date) else {
        return false;
    };

    first_interval.contains(&date)
}

fn next_change_from_intervals(
    date: NaiveDate,
    mut intervals: impl Iterator<Item = RangeInclusive<NaiveDate>>,
) -> NaiveDate {
    let Some(first_interval) = intervals.find(|rg| *rg.end() >= date) else {
        return DATE_END.date();
    };

    if *first_interval.start() <= date {
        first_interval.end().succ_opt().unwrap_or(DATE_END.date())
    } else {
        *first_interval.start()
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
            return Some(DATE_END.date());
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
        let (range, step) = self.into_parts();
        let year = Year(date.year());
        range.contains(&year) && (*year - **range.start()) % i32::from(step) == 0
    }

    fn next_change_hint<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        let (range, step) = self.into_parts();
        let curr_year = Year(date.year());

        let next_year = {
            if *range.end() < curr_year {
                // 1. time exceeded the range, the state won't ever change
                return Some(DATE_END.date());
            } else if curr_year < *range.start() {
                // 2. time didn't reach the range yet
                *range.start()
            } else if step == 1 {
                // 3. time is in the range and step is naive
                Year(**range.end() + 1)
            } else if (*curr_year - **range.start()) % i32::from(step) == 0 {
                // 4. time matches the range with step >= 2
                Year(*curr_year + 1)
            } else {
                // 5. time is in the range but doesn't match the step
                let round_up = |x: i32, d: i32| {
                    // get the first multiple of `d` greater than `x`.
                    debug_assert!(d > 0);
                    d * ((x + d - 1) / d)
                };

                Year(**range.start() + round_up(*curr_year - **range.start(), step.into()))
            }
        };

        Some(NaiveDate::from_ymd_opt(*next_year, 1, 1).unwrap_or(DATE_END.date()))
    }
}

/// Project date on a given year.
fn date_on_year(
    date: ds::Date,
    for_year: Year,
    date_builder: impl FnOnce(Year, u32, u32) -> NaiveDate,
) -> Option<NaiveDate> {
    match date {
        ds::Date::Easter { year } => easter(year.unwrap_or(for_year)),
        ds::Date::Fixed { year: None, month, day } => {
            Some(date_builder(for_year, month.into(), day.into()))
        }
        ds::Date::Fixed { year: Some(year), month, day } if year == for_year => {
            Some(date_builder(year, month.into(), day.into()))
        }
        _ => None,
    }
}

/// Transform a MonthdayRange into intervals that suround the input date
fn monthday_range_to_intervals(
    date: NaiveDate,
    monthday_range: &ds::MonthdayRange,
) -> Box<dyn Iterator<Item = RangeInclusive<NaiveDate>> + '_> {
    match monthday_range {
        ds::MonthdayRange::Month { range, year: None } => Box::new({
            let (range_start, range_end) = range.clone().into_inner();

            intervals_from_bounds(
                (date.year() - 1..=date.year() + 1)
                    .filter_map(move |y| NaiveDate::from_ymd_opt(y, range_start as _, 1)),
                (date.year() - 1..=date.year() + 1).filter_map(move |y| {
                    NaiveDate::from_ymd_opt(y, range_end.next() as _, 1)?.pred_opt()
                }),
            )
        }) as _,
        ds::MonthdayRange::Month { range, year: Some(year) } => Box::new({
            [**year, **year + 1]
                .into_iter()
                .filter_map(|end_year| {
                    let start = NaiveDate::from_ymd_opt(**year, *range.start() as _, 1)?;

                    let end = NaiveDate::from_ymd_opt(end_year, *range.end() as _, 1)?;

                    Some(start..=end)
                })
                .find(|rg| rg.start() <= rg.end())
                .into_iter()
        }) as _,
        ds::MonthdayRange::Date {
            start: (start, start_offset),
            end: (end, end_offset),
        } if start.year().is_some() || end.year().is_some() => {
            let year: Year = start.year().or(end.year()).unwrap();

            Box::new(
                [
                    (year, year),
                    (Year(*year - 1), year),
                    (year, Year(*year + 1)),
                ]
                .into_iter()
                .filter_map(|(year_start, year_end)| {
                    let year_start = start.year().unwrap_or(year_start);
                    let year_end = end.year().unwrap_or(year_end);

                    let start = date_on_year(*start, year_start, valid_ymd_after)
                        .map(|d| start_offset.apply(d))?;

                    let end = date_on_year(*end, year_end, valid_ymd_before)
                        .map(|d| end_offset.apply(d))?;

                    Some(start..=end)
                })
                .find(|rg| rg.start() <= rg.end())
                .into_iter(),
            ) as _
        }
        ds::MonthdayRange::Date {
            start: (start, start_offset),
            end: (end, end_offset),
        } => {
            let year = date.year();

            if *start == Date::md(29, Month::February) && *end == Date::md(29, Month::February) {
                return Box::new(
                    (year - 3..=year + 3)
                        .filter_map(|y| NaiveDate::from_ymd_opt(y, 2, 29))
                        .map(|d| start_offset.apply(d)..=end_offset.apply(d)),
                );
            }

            Box::new(intervals_from_bounds(
                (year - 1..=year + 1)
                    .filter_map(|y| date_on_year(*start, Year(y), valid_ymd_after))
                    .map(|d| start_offset.apply(d)),
                (year - 1..=year + 1)
                    .filter_map(|y| date_on_year(*end, Year(y), valid_ymd_before))
                    .map(|d| end_offset.apply(d)),
            )) as _
        }
    }
}

impl DateFilter for ds::MonthdayRange {
    fn filter<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> bool
    where
        L: Localize,
    {
        is_open_from_intervals(date, monthday_range_to_intervals(date, self))
    }

    fn next_change_hint<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        Some(next_change_from_intervals(
            date,
            monthday_range_to_intervals(date, self),
        ))
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

                range_u8.contains(&(date.weekday() as u8))
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
        let week = WeekNum(date.iso_week().week() as u8);
        let (range, step) = self.into_parts();

        range.contains(&week)
            // TODO: what happens when week < range.start ?
            && week.saturating_sub(range.start().0 ) % step == 0
    }

    fn next_change_hint<L>(&self, date: NaiveDate, _ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        let week = date.iso_week().week() as u8;
        let (range, step) = self.into_parts();

        if range.start() > range.end() {
            // TODO: wrapping implemented well?
            return None;
        }

        let weeknum = u32::from({
            if range.contains(&WeekNum(week)) {
                if step == 1 {
                    range.end().0 % 54 + 1
                } else if (week - range.start().0) % step == 0 {
                    (date.iso_week().week() as u8 % 54) + 1
                } else {
                    return None;
                }
            } else {
                range.start().0
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
