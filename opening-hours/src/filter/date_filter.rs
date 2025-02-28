use std::ops::RangeInclusive;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate};

use opening_hours_syntax::rules::day::{self as ds, Date, Month};

use crate::localization::Localize;
use crate::opening_hours::{DATE_END, DATE_START};
use crate::utils::dates::{count_days_in_month, easter};
use crate::utils::range::WrappingRange;
use crate::Context;

/// Get the first valid date before given "yyyy/mm/dd", for example if 2021/02/30 is given, this
/// will return february 28th as 2021 is not a leap year.
fn valid_ymd_before(year: i32, month: u32, day: u32) -> NaiveDate {
    debug_assert!((1..=31).contains(&day));

    NaiveDate::from_ymd_opt(year, month, day)
        .into_iter()
        .chain(
            (28..day)
                .rev()
                .filter_map(|day| NaiveDate::from_ymd_opt(year, month, day)),
        )
        .next()
        .unwrap_or(DATE_END.date())
}

/// Get the first valid date after given "yyyy/mm/dd", for example if 2021/02/30 is given, this
/// will return march 1st of 2021.
fn valid_ymd_after(year: i32, month: u32, day: u32) -> NaiveDate {
    debug_assert!((1..=31).contains(&day));

    NaiveDate::from_ymd_opt(year, month, day)
        .into_iter()
        .chain(
            (28..day)
                .rev()
                .filter_map(|day| NaiveDate::from_ymd_opt(year, month, day)?.succ_opt()),
        )
        .next()
        .unwrap_or(DATE_END.date())
}

// /// Find next change from iterators of "starting of an interval" to "end of an
// /// interval".
// fn next_change_from_bounds(
//     date: NaiveDate,
//     bounds_start: impl IntoIterator<Item = NaiveDate>,
//     bounds_end: impl IntoIterator<Item = NaiveDate>,
// ) -> NaiveDate {
//     next_change_from_intervals(date, intervals_from_bounds(bounds_start, bounds_end))
// }
//
// fn is_open_from_bounds(
//     date: NaiveDate,
//     bounds_start: impl IntoIterator<Item = NaiveDate>,
//     bounds_end: impl IntoIterator<Item = NaiveDate>,
// ) -> bool {
//     is_open_from_intervals(date, intervals_from_bounds(bounds_start, bounds_end))
// }

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

    std::iter::from_fn(move || {
        if let Some(start) = bounds_start.peek() {
            while bounds_end.next_if(|end| end < start).is_some() {}
        }

        let range = match (bounds_start.peek().copied(), bounds_end.peek().copied()) {
            // The date is after the end of the last interval
            (None, None) => return None,
            (None, Some(end)) => {
                bounds_end.next();
                DATE_START.date()..=end
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

    // Before the interval
    if *first_interval.start() > date {
        return *first_interval.start();
    }

    let mut intervals = intervals.peekable();

    let mut res = (first_interval.into_inner().1)
        .succ_opt()
        .unwrap_or(DATE_END.date());

    // Merge with following overlapping intervals
    while let Some(overlapping) = intervals.next_if(|next| *next.start() <= res) {
        res = std::cmp::max(
            res,
            (overlapping.into_inner().1)
                .succ_opt()
                .unwrap_or(DATE_END.date()),
        )
    }

    res
}

pub trait NewDateFilter {
    fn intervals<'a, L>(
        &'a self,
        date: NaiveDate,
        ctx: &'a Context<L>,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + 'a
    where
        L: Localize;
}

impl<T: NewDateFilter> DateFilter for T {
    fn filter<L>(&self, date: NaiveDate, ctx: &Context<L>) -> bool
    where
        L: Localize,
    {
        is_open_from_intervals(date, self.intervals(date, ctx))
    }

    fn next_change_hint<L>(&self, date: NaiveDate, ctx: &Context<L>) -> Option<NaiveDate>
    where
        L: Localize,
    {
        Some(next_change_from_intervals(date, self.intervals(date, ctx)))
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

impl NewDateFilter for ds::YearRange {
    fn intervals<'a, L>(
        &'a self,
        date: NaiveDate,
        _ctx: &Context<L>,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + 'a
    where
        L: Localize,
    {
        let (range, step) = self.into_parts();
        let (start, end) = range.into_inner();
        let [mut start, end, step] = [*start, *end, step].map(i32::from);

        if date.year() > start {
            // Find the first matching year in range start..=date.year()
            let nb_steps_skipped = (date.year() - start) / step;
            start += nb_steps_skipped * step;
        }

        (start..=end).step_by(step as _).filter_map(|y| {
            let start = NaiveDate::from_ymd_opt(y, 1, 1)?;
            let end = NaiveDate::from_ymd_opt(y, 12, 31)?;
            Some(start..=end)
        })
    }
}

/// Project date on a given year.
fn date_on_year(
    date: ds::Date,
    for_year: i32,
    date_builder: impl FnOnce(i32, u32, u32) -> NaiveDate,
) -> Option<NaiveDate> {
    match date {
        ds::Date::Easter { year } => easter(year.map(Into::into).unwrap_or(for_year)),
        ds::Date::Fixed { year: None, month, day } => {
            Some(date_builder(for_year, month.into(), day.into()))
        }
        ds::Date::Fixed { year: Some(year), month, day } if i32::from(year) == for_year => {
            Some(date_builder(year.into(), month.into(), day.into()))
        }
        _ => None,
    }
}

impl NewDateFilter for ds::MonthdayRange {
    fn intervals<'a, L>(
        &'a self,
        date: NaiveDate,
        _ctx: &'a Context<L>,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + 'a
    where
        L: Localize,
    {
        let year = date.year();

        let res: Box<dyn Iterator<Item = RangeInclusive<NaiveDate>>> = match self {
            ds::MonthdayRange::Month { range, year: None } => Box::new(intervals_from_bounds(
                (year - 1..DATE_END.date().year())
                    .filter_map(|y| NaiveDate::from_ymd_opt(y, *range.start() as _, 1)),
                ((year - 1)..DATE_END.date().year()).filter_map(|y| {
                    NaiveDate::from_ymd_opt(y, range.end().next() as _, 1)?.pred_opt()
                }),
            )) as _,
            ds::MonthdayRange::Month { range, year: Some(year) } => Box::new({
                [*year, year + 1]
                    .into_iter()
                    .filter_map(|end_year| {
                        let start =
                            NaiveDate::from_ymd_opt((*year).into(), *range.start() as _, 1)?;

                        let end = NaiveDate::from_ymd_opt(end_year.into(), *range.end() as _, 1)?;

                        Some(start..=end)
                    })
                    .find(|rg| rg.start() <= rg.end())
                    .into_iter()
            }) as _,
            ds::MonthdayRange::Date {
                start: (start, start_offset),
                end: (end, end_offset),
            } if start.year().is_some() || end.year().is_some() => Box::new({
                [(year, year), (year - 1, year), (year, year + 1)]
                    .into_iter()
                    .filter_map(|(year_start, year_end)| {
                        let year_start = start.year().map(Into::into).unwrap_or(year_start);
                        let year_end = end.year().map(Into::into).unwrap_or(year_end);

                        let start = date_on_year(*start, year_start, valid_ymd_after)
                            .map(|d| start_offset.apply(d))?;

                        let end = date_on_year(*end, year_end, valid_ymd_before)
                            .map(|d| end_offset.apply(d))?;

                        Some(start..=end)
                    })
                    .find(|rg| rg.start() <= rg.end())
                    .into_iter()
            }) as _,
            ds::MonthdayRange::Date {
                start: (start, start_offset),
                end: (end, end_offset),
            } => {
                // If both dates are Feb. 29th, then we shal only return a value for valid dates
                if [*start, *end] == [Date::md(29, Month::February); 2] {
                    return Box::new(
                        (year - 1..=DATE_END.year())
                            .filter_map(|y| NaiveDate::from_ymd_opt(y, 2, 29))
                            .map(|d| start_offset.apply(d)..=end_offset.apply(d)),
                    ) as _;
                }

                // Find all start and end bounds
                Box::new({
                    intervals_from_bounds(
                        (year - 1..=DATE_END.year()).filter_map(|year| {
                            date_on_year(*start, year, valid_ymd_after)
                                .map(|d| start_offset.apply(d))
                        }),
                        (year - 1..=DATE_END.year()).filter_map(|year| {
                            date_on_year(*end, year, valid_ymd_before).map(|d| end_offset.apply(d))
                        }),
                    )
                }) as _
            }
        };

        res
    }
}

impl NewDateFilter for ds::WeekDayRange {
    fn intervals<'a, L>(
        &'a self,
        date: NaiveDate,
        ctx: &'a Context<L>,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + 'a
    where
        L: Localize,
    {
        match self {
            ds::WeekDayRange::Fixed { range, offset, nth_from_start, nth_from_end } => Box::new({
                let (start, end) = range.clone().into_inner();

                let filter_by_position = |date: NaiveDate| -> bool {
                    let pos_from_start = (date.day() as u8 - 1) / 7;
                    let pos_from_end = (count_days_in_month(date) - date.day() as u8) / 7;

                    nth_from_start[usize::from(pos_from_start)]
                        || nth_from_end[usize::from(pos_from_end)]
                };

                let years_and_week = [(date.year() - 1, 52), (date.year() - 1, 53)]
                    .into_iter()
                    .chain(
                        (date.year()..DATE_END.year())
                            .flat_map(|year| (1..=53).map(move |weeknum| (year, weeknum))),
                    );

                intervals_from_bounds(
                    years_and_week.clone().filter_map(move |(year, weeknum)| {
                        NaiveDate::from_isoywd_opt(year, weeknum, start)
                    }),
                    years_and_week.filter_map(move |(year, weeknum)| {
                        NaiveDate::from_isoywd_opt(year, weeknum, end)
                    }),
                )
                .flat_map(|rg| {
                    let (mut curr, end) = rg.into_inner();

                    std::iter::from_fn(move || {
                        if curr > end {
                            return None;
                        }

                        let day = curr;
                        curr += Duration::days(1);
                        Some(day)
                    })
                })
                .filter(move |day| filter_by_position(*day))
                .map(|day| day + Duration::days(*offset))
                .map(|day| day..=day)
            })
                as _,
            ds::WeekDayRange::Holiday { kind, offset } => {
                let calendar = match kind {
                    ds::HolidayKind::Public => &ctx.holidays.public,
                    ds::HolidayKind::School => &ctx.holidays.school,
                };

                // TODO: jump to current year
                Box::new(
                    calendar
                        .iter()
                        .map(|d| d + Duration::days(*offset))
                        .map(|d| d..=d),
                ) as Box<dyn Iterator<Item = _>>
            }
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
