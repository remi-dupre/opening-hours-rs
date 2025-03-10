use std::ops::RangeInclusive;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate};

use opening_hours_syntax::rules::day::{self as ds, Date, Month, WeekNum, Year};
use opening_hours_syntax::rules::OpeningHoursExpression;

use crate::localization::Localize;
use crate::opening_hours::{DATE_END, DATE_START};
use crate::utils::dates::{count_days_in_month, easter};
use crate::Context;

pub(crate) fn dates_with_potential_change<L: Localize>(
    expr: &OpeningHoursExpression,
    ctx: &Context<L>,
    hint_start: NaiveDate,
    hint_end: NaiveDate,
) -> Box<dyn Iterator<Item = NaiveDate> + Send + Sync + 'static> {
    let res = expr.intervals(ctx, hint_start, hint_end).flat_map(|rg| {
        let (mut curr, end) = rg.into_inner();

        std::iter::from_fn(move || {
            if curr <= end {
                let res = curr;
                curr = curr.succ_opt()?;
                Some(res)
            } else {
                None
            }
        })
        .chain(end.succ_opt())
    });

    Box::new(ensure_increasing_iter(res)) as _
}

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

fn ensure_disjoint_ranges(
    iter: impl Iterator<Item = RangeInclusive<NaiveDate>>,
) -> impl Iterator<Item = RangeInclusive<NaiveDate>> {
    let mut iter = iter.peekable();

    std::iter::from_fn(move || {
        let (res_start, mut res_end) = iter.next()?.into_inner();

        while let Some(adj) = iter.next_if(|adj| *adj.start() <= res_end) {
            res_end = std::cmp::max(res_end, *adj.end());
        }

        Some(res_start..=res_end)
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

pub trait DateFilter {
    fn intervals<L>(
        &self,
        ctx: &Context<L>,
        hint_start: NaiveDate,
        hint_end: NaiveDate,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static
    where
        L: Localize;

    fn filter<L>(&self, date: NaiveDate, ctx: &Context<L>) -> bool
    where
        L: Localize,
    {
        is_open_from_intervals(date, self.intervals(ctx, date, date))
    }
}

impl DateFilter for OpeningHoursExpression {
    fn intervals<L>(
        &self,
        ctx: &Context<L>,
        hint_start: NaiveDate,
        hint_end: NaiveDate,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static
    where
        L: Localize,
    {
        self.rules.iter().fold(
            Box::new(std::iter::once(hint_start..=hint_end))
                as Box<dyn Iterator<Item = _> + Send + Sync + 'static>,
            |acc, x| {
                Box::new({
                    iter_range_union(acc, x.day_selector.intervals(ctx, hint_start, hint_end))
                })
            },
        )
    }
}

impl<T: DateFilter> DateFilter for [T] {
    fn intervals<L>(
        &self,
        ctx: &Context<L>,
        hint_start: NaiveDate,
        hint_end: NaiveDate,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static
    where
        L: Localize,
    {
        match self {
            [] => Box::new(std::iter::once(hint_start..=hint_end))
                as Box<dyn Iterator<Item = _> + Send + Sync + 'static>,
            [x] => Box::new(x.intervals(ctx, hint_start, hint_end)) as _,
            [x, y] => Box::new(iter_range_union(
                x.intervals(ctx, hint_start, hint_end),
                y.intervals(ctx, hint_start, hint_end),
            )) as _,
            full => Box::new({
                let (slice_a, slice_b) = full.split_at(full.len() / 2);
                iter_range_union(
                    slice_a.intervals(ctx, hint_start, hint_end),
                    slice_b.intervals(ctx, hint_start, hint_end),
                )
            }) as _,
        }
    }

    fn filter<L>(&self, date: NaiveDate, ctx: &Context<L>) -> bool
    where
        L: Localize,
    {
        self.is_empty() || self.iter().any(|x| x.filter(date, ctx))
    }
}

impl DateFilter for ds::DaySelector {
    fn intervals<L>(
        &self,
        ctx: &Context<L>,
        hint_start: NaiveDate,
        hint_end: NaiveDate,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static
    where
        L: Localize,
    {
        iter_range_intersection(
            iter_range_intersection(
                self.year.intervals(ctx, hint_start, hint_end),
                self.monthday.intervals(ctx, hint_start, hint_end),
            ),
            iter_range_intersection(
                self.week.intervals(ctx, hint_start, hint_end),
                self.weekday.intervals(ctx, hint_start, hint_end),
            ),
        )
    }

    fn filter<L>(&self, date: NaiveDate, ctx: &Context<L>) -> bool
    where
        L: Localize,
    {
        self.year.filter(date, ctx)
            && self.monthday.filter(date, ctx)
            && self.week.filter(date, ctx)
            && self.weekday.filter(date, ctx)
    }
}

impl DateFilter for ds::YearRange {
    fn intervals<L>(
        &self,
        _ctx: &Context<L>,
        hint_start: NaiveDate,
        hint_end: NaiveDate,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static
    where
        L: Localize,
    {
        let (range, step) = self.into_parts();
        let (Year(start), Year(end)) = range.into_inner();
        let [mut start, mut end, step] = [start, end, step].map(i32::from);

        if hint_start.year() > start {
            // Find the first matching year in range start..=date.year()
            let nb_steps_skipped = (hint_start.year() - start) / step;
            start += nb_steps_skipped * step;
        }

        if hint_end.year() < end {
            end = hint_end.year();
        }

        if step > 1 {
            Box::new({
                (start..=end).step_by(step as _).filter_map(|y| {
                    let start = NaiveDate::from_ymd_opt(y, 1, 1)?;
                    let end = NaiveDate::from_ymd_opt(y, 12, 31)?;
                    Some(start..=end)
                })
            }) as _
        } else {
            Box::new(
                NaiveDate::from_ymd_opt(start, 1, 1)
                    .and_then(|start| {
                        NaiveDate::from_ymd_opt(end, 12, 31)
                            .map(move |end| std::iter::once(start..=end))
                    })
                    .into_iter()
                    .flatten(),
            )
                as Box<dyn Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static>
        }
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

impl DateFilter for ds::MonthdayRange {
    fn intervals<L>(
        &self,
        _ctx: &Context<L>,
        hint_start: NaiveDate,
        hint_end: NaiveDate,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static
    where
        L: Localize,
    {
        let year = hint_start.year();

        let res: Box<dyn Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static> =
            match self {
                ds::MonthdayRange::Month { range, year: None } => Box::new({
                    let (range_start, range_end) = range.clone().into_inner();

                    intervals_from_bounds(
                        (year - 1..=hint_end.year())
                            .filter_map(move |y| NaiveDate::from_ymd_opt(y, range_start as _, 1)),
                        (year - 1..=hint_end.year()).filter_map(move |y| {
                            NaiveDate::from_ymd_opt(y, range_end.next() as _, 1)?.pred_opt()
                        }),
                    )
                }) as _,
                ds::MonthdayRange::Month { range, year: Some(year) } => Box::new({
                    [*year, year + 1]
                        .into_iter()
                        .filter_map(|end_year| {
                            let start =
                                NaiveDate::from_ymd_opt((*year).into(), *range.start() as _, 1)?;

                            let end =
                                NaiveDate::from_ymd_opt(end_year.into(), *range.end() as _, 1)?;

                            Some(start..=end)
                        })
                        .find(|rg| rg.start() <= rg.end())
                        .into_iter()
                }) as _,
                &ds::MonthdayRange::Date {
                    start: (start, start_offset),
                    end: (end, end_offset),
                } if start.year().is_some() || end.year().is_some() => Box::new({
                    // The output interval won't depend on the start hint, so we only base on the
                    // existing bound.
                    let year = start.year().or(end.year()).unwrap().into();

                    [(year, year), (year - 1, year), (year, year + 1)]
                        .into_iter()
                        .filter_map(|(year_start, year_end)| {
                            let year_start = start.year().map(Into::into).unwrap_or(year_start);
                            let year_end = end.year().map(Into::into).unwrap_or(year_end);

                            let start = date_on_year(start, year_start, valid_ymd_after)
                                .map(|d| start_offset.apply(d))?;

                            let end = date_on_year(end, year_end, valid_ymd_before)
                                .map(|d| end_offset.apply(d))?;

                            Some(start..=end)
                        })
                        .find(|rg| rg.start() <= rg.end())
                        .into_iter()
                }) as _,
                &ds::MonthdayRange::Date {
                    start: (start, start_offset),
                    end: (end, end_offset),
                } => {
                    // If both dates are Feb. 29th, then we shal only return a value for valid dates
                    if [start, end] == [Date::md(29, Month::February); 2] {
                        return Box::new(
                            (year - 1..=hint_end.year())
                                .filter_map(move |y| NaiveDate::from_ymd_opt(y, 2, 29))
                                .map(move |d| start_offset.apply(d)..=end_offset.apply(d)),
                        ) as _;
                    }

                    // Find all start and end bounds
                    Box::new({
                        intervals_from_bounds(
                            (year - 1..=hint_end.year()).filter_map(move |year| {
                                date_on_year(start, year, valid_ymd_after)
                                    .map(|d| start_offset.apply(d))
                            }),
                            (year - 1..=hint_end.year()).filter_map(move |year| {
                                date_on_year(end, year, valid_ymd_before)
                                    .map(|d| end_offset.apply(d))
                            }),
                        )
                    }) as _
                }
            };

        res
    }
}

impl DateFilter for ds::WeekDayRange {
    fn intervals<L>(
        &self,
        ctx: &Context<L>,
        hint_start: NaiveDate,
        hint_end: NaiveDate,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static
    where
        L: Localize,
    {
        match self.clone() {
            ds::WeekDayRange::Fixed { range, offset, nth_from_start, nth_from_end } => {
                let (start, end) = range.into_inner();

                let filter_by_position = move |date: NaiveDate| -> bool {
                    let pos_from_start = (date.day() as u8 - 1) / 7;
                    let pos_from_end = (count_days_in_month(date) - date.day() as u8) / 7;

                    nth_from_start[usize::from(pos_from_start)]
                        || nth_from_end[usize::from(pos_from_end)]
                };

                let years_and_week = [(hint_start.year() - 1, 52), (hint_start.year() - 1, 53)]
                    .into_iter()
                    .chain(
                        (hint_start.year()..=hint_end.year())
                            .flat_map(|year| (1..=53).map(move |weeknum| (year, weeknum))),
                    )
                    .chain(std::iter::once((hint_start.year() + 1, 1)));

                let base_intervals = intervals_from_bounds(
                    years_and_week.clone().filter_map(move |(year, weeknum)| {
                        NaiveDate::from_isoywd_opt(year, weeknum, start)
                    }),
                    years_and_week.filter_map(move |(year, weeknum)| {
                        NaiveDate::from_isoywd_opt(year, weeknum, end)
                    }),
                );

                if nth_from_start == [true; 5] && nth_from_end == [true; 5] {
                    Box::new(base_intervals.map(move |rg| {
                        let (start, end) = rg.into_inner();
                        let start = start + Duration::days(offset);
                        let end = end + Duration::days(offset);
                        start..=end
                    })) as Box<dyn Iterator<Item = _> + Send + Sync + 'static>
                } else {
                    Box::new(
                        base_intervals
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
                            .map(move |day| day + Duration::days(offset))
                            .map(|day| day..=day),
                    ) as _
                }
            }

            ds::WeekDayRange::Holiday { kind, offset } => {
                let calendar = match kind {
                    ds::HolidayKind::Public => ctx.holidays.public.clone(),
                    ds::HolidayKind::School => ctx.holidays.school.clone(),
                };

                let res = (hint_start.year()..=hint_end.year())
                    .filter_map(move |year| {
                        Some((
                            year,
                            *calendar.year_for(NaiveDate::from_ymd_opt(year, 1, 1)?)?,
                        ))
                    })
                    .flat_map(|(year, cal)| {
                        cal.into_iter().filter_map(move |(month, day)| {
                            NaiveDate::from_ymd_opt(year, month, day)
                        })
                    })
                    .map(move |d| d + Duration::days(offset))
                    .map(|d| d..=d);

                Box::new(res) as _
            }
        }
    }
}

impl DateFilter for ds::WeekRange {
    fn intervals<L>(
        &self,
        _ctx: &Context<L>,
        hint_start: NaiveDate,
        hint_end: NaiveDate,
    ) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static
    where
        L: Localize,
    {
        let (range, step) = self.into_parts();
        let (WeekNum(start), WeekNum(end)) = range.into_inner();

        let weeknums = {
            if start <= end {
                #[allow(clippy::reversed_empty_ranges)]
                (start..=end).chain(1..=0)
            } else {
                (start..=53).chain(1..=end)
            }
        };

        let weeknums = weeknums.step_by(step.into());

        (hint_start.year() - 1..=hint_end.year() + 1).flat_map(move |year| {
            weeknums.clone().filter_map(move |weeknum| {
                let start = NaiveDate::from_isoywd_opt(year, weeknum.into(), ds::Weekday::Mon)?;
                let end = NaiveDate::from_isoywd_opt(year, weeknum.into(), ds::Weekday::Sun)?;
                Some(start..=end)
            })
        })
    }
}

fn iter_range_union(
    iter_a: impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static,
    iter_b: impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static,
) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static {
    let mut iter_a = iter_a.peekable();
    let mut iter_b = iter_b.peekable();

    ensure_disjoint_ranges(std::iter::from_fn(move || {
        match (iter_a.peek(), iter_b.peek()) {
            (None, None) => None,
            (None, Some(_)) | (Some(_), None) => iter_a.next().or(iter_b.next()),
            (Some(a), Some(b)) => {
                if a.start() <= b.start() {
                    iter_a.next()
                } else {
                    iter_b.next()
                }
            }
        }
    }))
}

fn iter_range_intersection(
    iter_a: impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static,
    iter_b: impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static,
) -> impl Iterator<Item = RangeInclusive<NaiveDate>> + Send + Sync + 'static {
    let mut iter_a = iter_a.peekable();
    let mut iter_b = iter_b.peekable();

    ensure_disjoint_ranges(std::iter::from_fn(move || {
        let mut from_a = iter_a.peek()?;
        let mut from_b = iter_b.peek()?;

        while std::cmp::max(from_a.start(), from_b.start())
            > std::cmp::min(from_a.end(), from_b.end())
        {
            if from_a.start() <= from_b.start() {
                iter_a.next();
                from_a = iter_a.peek()?;
            } else {
                iter_b.next();
                from_b = iter_b.peek()?;
            }
        }

        let start = *std::cmp::max(from_a.start(), from_b.start());
        let end = *std::cmp::min(from_a.end(), from_b.end());

        if end >= *from_a.end() {
            iter_a.next();
        }

        if end >= *from_b.end() {
            iter_b.next();
        }

        Some(start..=end)
    }))
}
