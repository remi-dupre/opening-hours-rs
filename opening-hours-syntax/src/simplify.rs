#![allow(clippy::single_range_in_vec_init)]
use std::iter::{Chain, Once};
use std::ops::Range;
use std::sync::Arc;

use crate::rubik::{Paving, Paving5D, PavingSelector, Selector4D, Selector5D};
use crate::rules::day::{DaySelector, MonthdayRange, WeekDayRange, WeekRange, YearRange};
use crate::rules::time::{Time, TimeSelector, TimeSpan};
use crate::rules::{RuleOperator, RuleSequence};
use crate::sorted_vec::UniqueSortedVec;
use crate::{ExtendedTime, RuleKind};

pub(crate) type Canonical = Paving5D<ExtendedTime, u8, u8, u8, u16>;

pub(crate) const FULL_YEARS: Range<u16> = 1900..10_000;
pub(crate) const FULL_MONTHDAYS: Range<u8> = 1..13;
pub(crate) const FULL_WEEKS: Range<u8> = 1..54;
pub(crate) const FULL_WEEKDAY: Range<u8> = 0..7;
pub(crate) const FULL_TIME: Range<ExtendedTime> =
    ExtendedTime::new(0, 0).unwrap()..ExtendedTime::new(48, 0).unwrap();

enum OneOrTwo<T> {
    One(T),
    Two(T, T),
}

impl<T> OneOrTwo<T> {
    fn map<U>(self, mut func: impl FnMut(T) -> U) -> OneOrTwo<U> {
        match self {
            OneOrTwo::One(x) => OneOrTwo::One(func(x)),
            OneOrTwo::Two(x, y) => OneOrTwo::Two(func(x), func(y)),
        }
    }
}

impl<T> IntoIterator for OneOrTwo<T> {
    type Item = T;
    type IntoIter = Chain<Once<T>, <Option<T> as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            OneOrTwo::One(x) => std::iter::once(x).chain(None),
            OneOrTwo::Two(x, y) => std::iter::once(x).chain(Some(y)),
        }
    }
}

// Ensure that input range is "increasing", otherwise it is splited into two ranges:
// [bounds.start, range.end[ and [range.start, bounds.end[
fn split_inverted_range<T: Ord>(range: Range<T>, bounds: Range<T>) -> OneOrTwo<Range<T>> {
    if range.start >= range.end {
        // start == end when a wrapping range gets expanded from exclusive to inclusive range
        OneOrTwo::Two(bounds.start..range.end, range.start..bounds.end)
    } else {
        OneOrTwo::One(range)
    }
}

fn vec_with_default<T>(default: T, mut vec: Vec<T>) -> Vec<T> {
    if vec.is_empty() {
        vec.push(default);
    }

    vec
}

pub(crate) fn ruleseq_to_day_selector(rs: &RuleSequence) -> Option<Selector4D<u8, u8, u8, u16>> {
    let ds = &rs.day_selector;

    let selector = PavingSelector::empty()
        .dim(vec_with_default(
            FULL_YEARS,
            (ds.year.iter())
                .flat_map(|year| {
                    if year.step != 1 {
                        return OneOrTwo::One(None);
                    }

                    let start = *year.range.start();
                    let end = *year.range.end() + 1;
                    split_inverted_range(start..end, FULL_YEARS).map(Some)
                })
                .collect::<Option<Vec<_>>>()?,
        ))
        .dim(vec_with_default(
            FULL_MONTHDAYS,
            (ds.monthday.iter())
                .flat_map(|monthday| match monthday {
                    MonthdayRange::Month { range, year: None } => {
                        let start = *range.start() as u8;
                        let end = *range.end() as u8 + 1;
                        split_inverted_range(start..end, FULL_MONTHDAYS).map(Some)
                    }
                    _ => OneOrTwo::One(None),
                })
                .collect::<Option<Vec<_>>>()?,
        ))
        .dim(vec_with_default(
            FULL_WEEKS,
            (ds.week.iter())
                .flat_map(|week| {
                    if week.step != 1 {
                        return OneOrTwo::One(None);
                    }

                    let start = *week.range.start();
                    let end = *week.range.end() + 1;
                    split_inverted_range(start..end, FULL_WEEKS).map(Some)
                })
                .collect::<Option<Vec<_>>>()?,
        ))
        .dim(vec_with_default(
            FULL_WEEKDAY,
            (ds.weekday.iter())
                .flat_map(|weekday| {
                    match weekday {
                        WeekDayRange::Fixed {
                            range,
                            offset: 0,
                            nth_from_start: [true, true, true, true, true], // TODO: could be canonical
                            nth_from_end: [true, true, true, true, true], // TODO: could be canonical
                        } => {
                            let start = *range.start() as u8;
                            let end = *range.end() as u8 + 1;
                            split_inverted_range(start..end, FULL_WEEKDAY).map(Some)
                        }
                        _ => OneOrTwo::One(None),
                    }
                })
                .collect::<Option<Vec<_>>>()?,
        ));

    Some(selector)
}

pub(crate) fn ruleseq_to_selector(
    rs: &RuleSequence,
) -> Option<Selector5D<ExtendedTime, u8, u8, u8, u16>> {
    Some(
        ruleseq_to_day_selector(rs)?.dim(vec_with_default(
            FULL_TIME,
            (rs.time_selector.time.iter())
                .flat_map(|time| match time {
                    TimeSpan { range, open_end: false, repeats: None } => {
                        let Time::Fixed(start) = range.start else {
                            return OneOrTwo::One(None);
                        };

                        let Time::Fixed(end) = range.end else {
                            return OneOrTwo::One(None);
                        };

                        split_inverted_range(start..end, FULL_TIME).map(Some)
                    }
                    _ => OneOrTwo::One(None),
                })
                .collect::<Option<Vec<_>>>()?,
        )),
    )
}

pub(crate) fn canonical_to_seq(
    mut canonical: Canonical,
    operator: RuleOperator,
    kind: RuleKind,
    comments: UniqueSortedVec<Arc<str>>,
) -> impl Iterator<Item = RuleSequence> {
    std::iter::from_fn(move || {
        let selector = canonical.pop_selector()?;
        let (rgs_time, selector) = selector.unpack();
        let (rgs_weekday, selector) = selector.unpack();
        let (rgs_week, selector) = selector.unpack();
        let (rgs_monthday, selector) = selector.unpack();
        let (rgs_year, _) = selector.unpack();

        let day_selector = DaySelector {
            year: (rgs_year.iter())
                .filter(|rg| **rg != FULL_YEARS)
                .map(|rg_year| YearRange { range: rg_year.start..=rg_year.end - 1, step: 1 })
                .collect(),
            monthday: (rgs_monthday.iter())
                .filter(|rg| **rg != FULL_MONTHDAYS)
                .map(|rg_month| MonthdayRange::Month {
                    range: rg_month.start.try_into().expect("invalid starting month")
                        ..=(rg_month.end - 1).try_into().expect("invalid ending month"),
                    year: None,
                })
                .collect(),
            week: (rgs_week.iter())
                .filter(|rg| **rg != FULL_WEEKS)
                .map(|rg_week| WeekRange { range: rg_week.start..=rg_week.end - 1, step: 1 })
                .collect(),
            weekday: (rgs_weekday.iter())
                .filter(|rg| **rg != FULL_WEEKDAY)
                .map(|rg_weekday| WeekDayRange::Fixed {
                    range: (rg_weekday.start).try_into().expect("invalid starting day")
                        ..=(rg_weekday.end - 1).try_into().expect("invalid ending day"),
                    offset: 0,
                    nth_from_start: [true; 5],
                    nth_from_end: [true; 5],
                })
                .collect(),
        };

        let time_selector = TimeSelector {
            time: (rgs_time.iter())
                .filter(|rg| **rg != FULL_TIME)
                .map(|rg_time| TimeSpan {
                    range: Time::Fixed(rg_time.start)..Time::Fixed(rg_time.end),
                    open_end: false,
                    repeats: None,
                })
                .collect(),
        };

        Some(RuleSequence {
            day_selector,
            time_selector,
            kind,
            operator,
            comments: comments.clone(),
        })
    })
}
