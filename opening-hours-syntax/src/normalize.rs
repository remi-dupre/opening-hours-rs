#![allow(clippy::single_range_in_vec_init)]
use std::cmp::Ordering;
use std::iter::{Chain, Once};
use std::ops::{Range, RangeInclusive};
use std::sync::Arc;

use chrono::Weekday;

use crate::rubik::{Paving, Paving5D, PavingSelector, Selector4D, Selector5D};
use crate::rules::day::{
    DaySelector, Month, MonthdayRange, WeekDayRange, WeekNum, WeekRange, Year, YearRange,
};
use crate::rules::time::{Time, TimeSelector, TimeSpan};
use crate::rules::{RuleOperator, RuleSequence};
use crate::sorted_vec::UniqueSortedVec;
use crate::{ExtendedTime, RuleKind};

pub(crate) type Canonical =
    Paving5D<ExtendedTime, Frame<OrderedWeekday>, Frame<WeekNum>, Frame<Month>, Frame<Year>>;

pub(crate) type CanonicalSelector =
    Selector5D<ExtendedTime, Frame<OrderedWeekday>, Frame<WeekNum>, Frame<Month>, Frame<Year>>;

// --
// -- OneOrTwo
// --

pub(crate) enum OneOrTwo<T> {
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

// --
// -- OrderedWeekday
// ---

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct OrderedWeekday(Weekday);

impl Ord for OrderedWeekday {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .number_from_monday()
            .cmp(&other.0.number_from_monday())
    }
}

impl PartialOrd for OrderedWeekday {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<OrderedWeekday> for Weekday {
    fn from(val: OrderedWeekday) -> Self {
        val.0
    }
}

impl From<Weekday> for OrderedWeekday {
    fn from(value: Weekday) -> Self {
        Self(value)
    }
}

// --
// -- Framable
// --

pub(crate) trait Framable: PartialEq + Eq + PartialOrd + Ord {
    const FRAME_START: Self;
    const FRAME_END: Self;

    fn succ(self) -> Self;
    fn pred(self) -> Self;
}

impl Framable for OrderedWeekday {
    const FRAME_START: Self = OrderedWeekday(Weekday::Mon);
    const FRAME_END: Self = OrderedWeekday(Weekday::Sun);

    fn succ(self) -> Self {
        OrderedWeekday(self.0.succ())
    }

    fn pred(self) -> Self {
        OrderedWeekday(self.0.pred())
    }
}

impl Framable for Month {
    const FRAME_START: Self = Month::January;
    const FRAME_END: Self = Month::December;

    fn succ(self) -> Self {
        self.next()
    }

    fn pred(self) -> Self {
        self.prev()
    }
}

impl Framable for Year {
    const FRAME_START: Self = Year(1900);
    const FRAME_END: Self = Year(9999);

    fn succ(self) -> Self {
        Year(self.0 + 1)
    }

    fn pred(self) -> Self {
        Year(self.0 - 1)
    }
}

impl Framable for WeekNum {
    const FRAME_START: Self = WeekNum(1);
    const FRAME_END: Self = WeekNum(53);

    fn succ(self) -> Self {
        WeekNum(*self % 53 + 1)
    }

    fn pred(self) -> Self {
        WeekNum((*self + 51) % 53 + 1)
    }
}

// --
// -- Frame
// --

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Frame<T: Framable> {
    Val(T),
    End,
}

impl<T: Framable> Frame<T> {
    fn to_range_strict(range: RangeInclusive<T>) -> Range<Frame<T>> {
        let (start, end) = range.into_inner();

        let strict_end = {
            if end == T::FRAME_END {
                Frame::End
            } else {
                Frame::Val(end.succ())
            }
        };

        Self::Val(start)..strict_end
    }

    fn to_range_inclusive(range: Range<Frame<T>>) -> Option<RangeInclusive<T>> {
        match (range.start, range.end) {
            (Frame::Val(x), Frame::Val(y)) => Some(x..=y.pred()),
            (Frame::Val(x), Frame::End) => Some(x..=T::FRAME_END),
            (Frame::End, Frame::Val(y)) => Some(T::FRAME_END..=y.pred()),
            (Frame::End, Frame::End) => None,
        }
    }
}

impl<T: Framable> PartialOrd for Frame<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Framable> Ord for Frame<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Frame::Val(x), Frame::Val(y)) => x.cmp(y),
            (Frame::Val(_), Frame::End) => Ordering::Less,
            (Frame::End, Frame::Val(_)) => Ordering::Greater,
            (Frame::End, Frame::End) => Ordering::Equal,
        }
    }
}

// --
// -- Bounded
// --

pub(crate) trait Bounded: Ord + Sized {
    const BOUND_START: Self;
    const BOUND_END: Self; // Excluded

    fn bounds() -> Range<Self> {
        Self::BOUND_START..Self::BOUND_END
    }

    // Ensure that input range is "increasing", otherwise it is splited into two ranges:
    // [bounds.start, range.end[ and [range.start, bounds.end[
    fn split_inverted_range(range: Range<Self>) -> OneOrTwo<Range<Self>> {
        if range.start >= range.end {
            // start == end when a wrapping range gets expanded from exclusive to inclusive range
            OneOrTwo::Two(Self::BOUND_START..range.end, range.start..Self::BOUND_END)
        } else {
            OneOrTwo::One(range)
        }
    }
}

impl<T: Framable> Bounded for Frame<T> {
    const BOUND_START: Self = Frame::Val(T::FRAME_START);
    const BOUND_END: Self = Frame::End;
}

impl Bounded for ExtendedTime {
    const BOUND_START: Self = ExtendedTime::new(0, 0).unwrap();
    const BOUND_END: Self = ExtendedTime::new(48, 0).unwrap();
}

// --
// -- Normalization Logic
// --

fn vec_with_default<T>(default: T, mut vec: Vec<T>) -> Vec<T> {
    if vec.is_empty() {
        vec.push(default);
    }

    vec
}

pub(crate) fn ruleseq_to_day_selector(
    rs: &RuleSequence,
) -> Option<Selector4D<Frame<OrderedWeekday>, Frame<WeekNum>, Frame<Month>, Frame<Year>>> {
    let ds = &rs.day_selector;

    let selector = PavingSelector::empty()
        .dim(vec_with_default(
            Bounded::bounds(),
            (ds.year.iter())
                .flat_map(|year| {
                    if year.step != 1 {
                        return OneOrTwo::One(None);
                    }

                    Bounded::split_inverted_range(Frame::to_range_strict(year.range.clone()))
                        .map(Some)
                })
                .collect::<Option<Vec<_>>>()?,
        ))
        .dim(vec_with_default(
            Bounded::bounds(),
            (ds.monthday.iter())
                .flat_map(|monthday| match monthday {
                    MonthdayRange::Month { range, year: None } => {
                        Bounded::split_inverted_range(Frame::to_range_strict(range.clone()))
                            .map(Some)
                    }
                    _ => OneOrTwo::One(None),
                })
                .collect::<Option<Vec<_>>>()?,
        ))
        .dim(vec_with_default(
            Bounded::bounds(),
            (ds.week.iter())
                .flat_map(|week| {
                    if week.step != 1 {
                        return OneOrTwo::One(None);
                    }

                    Bounded::split_inverted_range(Frame::to_range_strict(week.range.clone()))
                        .map(Some)
                })
                .collect::<Option<Vec<_>>>()?,
        ))
        .dim(vec_with_default(
            Bounded::bounds(),
            (ds.weekday.iter())
                .flat_map(|weekday| {
                    match weekday {
                        WeekDayRange::Fixed {
                            range,
                            offset: 0,
                            // NOTE: These could be turned into canonical
                            // dimensions, but it may be uncommon enough to
                            // avoid extra complexity.
                            nth_from_start: [true, true, true, true, true],
                            nth_from_end: [true, true, true, true, true],
                        } => {
                            let (start, end) = range.clone().into_inner();

                            Bounded::split_inverted_range(Frame::to_range_strict(
                                start.into()..=end.into(),
                            ))
                            .map(Some)
                        }
                        _ => OneOrTwo::One(None),
                    }
                })
                .collect::<Option<Vec<_>>>()?,
        ));

    Some(selector)
}

pub(crate) fn ruleseq_to_selector(rs: &RuleSequence) -> Option<CanonicalSelector> {
    Some(
        ruleseq_to_day_selector(rs)?.dim(vec_with_default(
            Bounded::bounds(),
            (rs.time_selector.time.iter())
                .flat_map(|time| match time {
                    TimeSpan { range, open_end: false, repeats: None } => {
                        let Time::Fixed(start) = range.start else {
                            return OneOrTwo::One(None);
                        };

                        let Time::Fixed(end) = range.end else {
                            return OneOrTwo::One(None);
                        };

                        Bounded::split_inverted_range(start..end).map(Some)
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
                .filter(|rg| **rg != Bounded::bounds())
                .filter_map(|rg_year| {
                    Some(YearRange {
                        range: Frame::to_range_inclusive(rg_year.clone())?,
                        step: 1,
                    })
                })
                .collect(),
            monthday: (rgs_monthday.iter())
                .filter(|rg| **rg != Bounded::bounds())
                .filter_map(|rg_month| {
                    Some(MonthdayRange::Month {
                        range: Frame::to_range_inclusive(rg_month.clone())?,
                        year: None,
                    })
                })
                .collect(),
            week: (rgs_week.iter())
                .filter(|rg| **rg != Bounded::bounds())
                .filter_map(|rg_week| {
                    Some(WeekRange {
                        range: Frame::to_range_inclusive(rg_week.clone())?,
                        step: 1,
                    })
                })
                .collect(),
            weekday: (rgs_weekday.iter())
                .filter(|rg| **rg != Bounded::bounds())
                .filter_map(|rg_weekday| {
                    let (start, end) = Frame::to_range_inclusive(rg_weekday.clone())?.into_inner();

                    Some(WeekDayRange::Fixed {
                        range: start.into()..=end.into(),
                        offset: 0,
                        nth_from_start: [true; 5],
                        nth_from_end: [true; 5],
                    })
                })
                .collect(),
        };

        let time_selector = TimeSelector {
            time: (rgs_time.iter())
                .filter(|rg| **rg != Bounded::bounds())
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
