#![allow(clippy::single_range_in_vec_init)]
use std::cmp::Ordering;
use std::ops::{Range, RangeInclusive};
use std::sync::Arc;

use chrono::Weekday;

use crate::rubik::{DimFromBack, EmptyPavingSelector, Paving, Paving4D, Paving5D, Selector5D};
use crate::rules::day::{
    DaySelector, Month, MonthdayRange, WeekDayRange, WeekNum, WeekRange, Year, YearRange,
};
use crate::rules::time::{Time, TimeSelector, TimeSpan};
use crate::rules::{RuleOperator, RuleSequence};
use crate::sorted_vec::UniqueSortedVec;
use crate::{ExtendedTime, RuleKind};

pub(crate) type Canonical = Paving5D<
    Frame<Year>,
    Frame<Month>,
    Frame<WeekNum>,
    Frame<OrderedWeekday>,
    ExtendedTime,
    (RuleKind, UniqueSortedVec<Arc<str>>),
>;

pub(crate) type CanonicalSelector =
    Selector5D<Frame<Year>, Frame<Month>, Frame<WeekNum>, Frame<OrderedWeekday>, ExtendedTime>;

// --
// -- OrderedWeekday
// ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
    fn split_inverted_range(range: Range<Self>) -> impl Iterator<Item = Range<Self>> {
        if range.start >= range.end {
            // start == end when a wrapping range gets expanded from exclusive to inclusive range
            std::iter::once(Self::BOUND_START..range.end).chain(Some(range.start..Self::BOUND_END))
        } else {
            std::iter::once(range).chain(None)
        }
    }
}

impl<T: Framable> Bounded for Frame<T> {
    const BOUND_START: Self = Frame::Val(T::FRAME_START);
    const BOUND_END: Self = Frame::End;
}

impl Bounded for ExtendedTime {
    // TODO: bounds to 48 could be handled but it's kinda tricky in current form
    // (eg. "Feb ; 18:00-28:00 closed" has to be something like "Feb1 00:00-18:00 ; Feb2-Feb29
    // 04:00-18:00").
    // To solve that, the time should probably not be a dimension at all?
    const BOUND_START: Self = ExtendedTime::new(0, 0).unwrap();
    const BOUND_END: Self = ExtendedTime::new(24, 0).unwrap();
}

// --
// -- MakeCanonical
// --

trait MakeCanonical: Sized + 'static {
    type CanonicalType: Bounded;
    fn try_make_canonical(&self) -> Option<Range<Self::CanonicalType>>;
    fn into_type(canonical: Range<Self::CanonicalType>) -> Option<Self>;

    fn try_from_iterator<'a>(
        iter: impl IntoIterator<Item = &'a Self>,
    ) -> Option<Vec<Range<Self::CanonicalType>>> {
        let mut ranges = Vec::new();

        for elem in iter {
            let range = Self::try_make_canonical(elem)?;
            ranges.extend(Bounded::split_inverted_range(range));
        }

        if ranges.is_empty() {
            ranges.push(Self::CanonicalType::bounds())
        }

        Some(ranges)
    }

    fn into_selector(
        canonical: Vec<Range<Self::CanonicalType>>,
        remove_full_ranges: bool,
    ) -> Vec<Self> {
        canonical
            .into_iter()
            .filter(|rg| !(remove_full_ranges && *rg == Self::CanonicalType::bounds()))
            .filter_map(|rg| Self::into_type(rg))
            .collect()
    }
}

impl MakeCanonical for YearRange {
    type CanonicalType = Frame<Year>;

    fn try_make_canonical(&self) -> Option<Range<Self::CanonicalType>> {
        if self.step != 1 {
            return None;
        }

        Some(Frame::to_range_strict(self.range.clone()))
    }

    fn into_type(canonical: Range<Self::CanonicalType>) -> Option<Self> {
        Some(YearRange {
            range: Frame::to_range_inclusive(canonical)?,
            step: 1,
        })
    }
}

impl MakeCanonical for MonthdayRange {
    type CanonicalType = Frame<Month>;

    fn try_make_canonical(&self) -> Option<Range<Self::CanonicalType>> {
        match self {
            Self::Month { range, year: None } => Some(Frame::to_range_strict(range.clone())),
            _ => None,
        }
    }

    fn into_type(canonical: Range<Self::CanonicalType>) -> Option<Self> {
        Some(MonthdayRange::Month {
            range: Frame::to_range_inclusive(canonical)?,
            year: None,
        })
    }
}

impl MakeCanonical for WeekRange {
    type CanonicalType = Frame<WeekNum>;

    fn try_make_canonical(&self) -> Option<Range<Self::CanonicalType>> {
        if self.step != 1 {
            return None;
        }

        Some(Frame::to_range_strict(self.range.clone()))
    }

    fn into_type(canonical: Range<Self::CanonicalType>) -> Option<Self> {
        Some(WeekRange {
            range: Frame::to_range_inclusive(canonical)?,
            step: 1,
        })
    }
}

impl MakeCanonical for WeekDayRange {
    type CanonicalType = Frame<OrderedWeekday>;

    fn try_make_canonical(&self) -> Option<Range<Self::CanonicalType>> {
        match self {
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
                Some(Frame::to_range_strict(start.into()..=end.into()))
            }
            _ => None,
        }
    }

    fn into_type(canonical: Range<Self::CanonicalType>) -> Option<Self> {
        let (start, end) = Frame::to_range_inclusive(canonical)?.into_inner();

        Some(WeekDayRange::Fixed {
            range: start.into()..=end.into(),
            offset: 0,
            nth_from_start: [true; 5],
            nth_from_end: [true; 5],
        })
    }
}

impl MakeCanonical for TimeSpan {
    type CanonicalType = ExtendedTime;

    fn try_make_canonical(&self) -> Option<Range<Self::CanonicalType>> {
        match self {
            TimeSpan { range, open_end: false, repeats: None } => {
                let Time::Fixed(start) = range.start else {
                    return None;
                };

                let Time::Fixed(end) = range.end else {
                    return None;
                };

                if start >= end || end > ExtendedTime::BOUND_END {
                    return None;
                }

                Some(start..end)
            }
            _ => None,
        }
    }

    fn into_type(canonical: Range<Self::CanonicalType>) -> Option<Self> {
        Some(TimeSpan {
            range: Time::Fixed(canonical.start)..Time::Fixed(canonical.end),
            open_end: false,
            repeats: None,
        })
    }
}

// --
// -- Normalization Logic
// --

pub(crate) fn ruleseq_to_selector(rs: &RuleSequence) -> Option<CanonicalSelector> {
    let ds = &rs.day_selector;

    let selector = EmptyPavingSelector
        .dim_front(MakeCanonical::try_from_iterator(&rs.time_selector.time)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.weekday)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.week)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.monthday)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.year)?);

    Some(selector)
}

pub(crate) fn canonical_to_seq(mut canonical: Canonical) -> impl Iterator<Item = RuleSequence> {
    // Keep track of the days that have already been outputed. This allows to use an additional
    // rule if it is absolutly required only.
    let mut days_covered = Paving4D::default();

    std::iter::from_fn(move || {
        // Extract open periods first, then unknowns
        let ((kind, comments), selector) = [RuleKind::Open, RuleKind::Unknown, RuleKind::Closed]
            .into_iter()
            .find_map(|target_kind| {
                canonical.pop_filter(|(kind, comments)| {
                    *kind == target_kind
                        && (target_kind != RuleKind::Closed || !comments.is_empty())
                })
            })?;

        let (rgs_time, day_selector) = selector.into_unpack_back();

        let operator = {
            if days_covered.is_val(&day_selector, &false) {
                RuleOperator::Normal
            } else {
                RuleOperator::Additional
            }
        };

        days_covered.set(&day_selector, &true);
        let (rgs_year, day_selector) = day_selector.into_unpack_front();
        let (rgs_monthday, day_selector) = day_selector.into_unpack_front();
        let (rgs_week, day_selector) = day_selector.into_unpack_front();
        let (rgs_weekday, _) = day_selector.into_unpack_front();

        let day_selector = DaySelector {
            year: MakeCanonical::into_selector(rgs_year, true),
            monthday: MakeCanonical::into_selector(rgs_monthday, true),
            week: MakeCanonical::into_selector(rgs_week, true),
            weekday: MakeCanonical::into_selector(rgs_weekday, true),
        };

        let time_selector = TimeSelector {
            time: MakeCanonical::into_selector(rgs_time, false),
        };

        Some(RuleSequence {
            day_selector,
            time_selector,
            kind,
            operator,
            comments,
        })
    })
}
