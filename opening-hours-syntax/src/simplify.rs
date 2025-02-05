#![allow(clippy::single_range_in_vec_init)]
use std::ops::Range;
use std::sync::Arc;

use crate::rubik::{Paving, Paving5D, PavingSelector};
use crate::rules::day::{DaySelector, MonthdayRange, WeekDayRange, WeekRange, YearRange};
use crate::rules::time::{Time, TimeSelector, TimeSpan};
use crate::rules::{RuleOperator, RuleSequence};
use crate::sorted_vec::UniqueSortedVec;
use crate::{ExtendedTime, RuleKind};

pub(crate) type Canonical = Paving5D<ExtendedTime, u8, u8, u8, u16>;

const FULL_YEARS: Range<u16> = u16::MIN..u16::MAX;
const FULL_MONTHDAYS: Range<u8> = 1..13;
const FULL_WEEKS: Range<u8> = 1..6;
const FULL_WEEKDAY: Range<u8> = 0..7;
const FULL_TIME: Range<ExtendedTime> =
    ExtendedTime::new(0, 0).unwrap()..ExtendedTime::new(48, 0).unwrap();

fn vec_with_default<T>(default: T, mut vec: Vec<T>) -> Vec<T> {
    if vec.is_empty() {
        vec.push(default);
    }

    vec
}

pub(crate) fn seq_to_canonical(rs: &RuleSequence) -> Option<Canonical> {
    let ds = &rs.day_selector;
    let ts = &rs.time_selector;

    let selector = PavingSelector::empty()
        .dim(vec_with_default(
            FULL_YEARS,
            (ds.year.iter())
                .map(|year| {
                    if year.step != 1 {
                        return None;
                    }

                    let start = *year.range.start();
                    let end = *year.range.end() + 1;
                    Some(start..end)
                })
                .collect::<Option<Vec<_>>>()?,
        ))
        .dim(vec_with_default(
            FULL_MONTHDAYS,
            (ds.monthday.iter())
                .map(|monthday| match monthday {
                    MonthdayRange::Month { range, year: None } => {
                        let start = *range.start() as u8;
                        let end = *range.end() as u8 + 1;
                        Some(start..end)
                    }
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?,
        ))
        .dim(vec_with_default(
            FULL_WEEKS,
            (ds.week.iter())
                .map(|week| {
                    if week.step != 1 {
                        return None;
                    }

                    let start = *week.range.start();
                    let end = *week.range.end() + 1;
                    Some(start..end)
                })
                .collect::<Option<Vec<_>>>()?,
        ))
        .dim(vec_with_default(
            FULL_WEEKDAY,
            (ds.weekday.iter())
                .map(|weekday| {
                    match weekday {
                        WeekDayRange::Fixed {
                            range,
                            offset: 0,
                            nth_from_start: [true, true, true, true, true], // TODO: could be canonical
                            nth_from_end: [true, true, true, true, true], // TODO: could be canonical
                        } => {
                            let start = *range.start() as u8;
                            let end = *range.end() as u8 + 1;
                            Some(start..end)
                        }
                        _ => None,
                    }
                })
                .collect::<Option<Vec<_>>>()?,
        ))
        .dim(vec_with_default(
            FULL_TIME,
            (ts.time.iter())
                .map(|time| match time {
                    TimeSpan { range, open_end: false, repeats: None } => {
                        let Time::Fixed(start) = range.start else {
                            return None;
                        };

                        let Time::Fixed(end) = range.end else {
                            return None;
                        };

                        Some(start..end)
                    }
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?,
        ));

    let mut result = Paving5D::default();
    result.set(&dbg!(selector), true);
    Some(dbg!(result))
}

pub(crate) fn canonical_to_seq(
    mut canonical: Canonical,
    operator: RuleOperator,
    kind: RuleKind,
    comments: UniqueSortedVec<Arc<str>>,
) -> impl Iterator<Item = RuleSequence> {
    std::iter::from_fn(move || {
        let selector = dbg!(canonical.pop_selector())?;
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
