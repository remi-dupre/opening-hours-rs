//! Helpers to convert from open-ended ranges from and to close-ended ranges.

// --
// -- Framable
// --

use std::cmp::Ordering;
use std::ops::{Range, RangeInclusive};

use chrono::Weekday;

use crate::rules::day::{Month, WeekNum, Year};
use crate::ExtendedTime;

use super::canonical::OrderedWeekday;

/// A type that can be enclosed in a `Frame`.
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

/// Wraps an unbounded type to add a virtual bound end.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Frame<T: Framable> {
    Val(T),
    End,
}

impl<T: Framable> Frame<T> {
    pub(crate) fn to_range_strict(range: RangeInclusive<T>) -> Range<Frame<T>> {
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

    pub(crate) fn to_range_inclusive(range: Range<Frame<T>>) -> Option<RangeInclusive<T>> {
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

/// A type with a lower bound and an open ended bound.
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
    const BOUND_START: Self = ExtendedTime::MIDNIGHT_00;
    const BOUND_END: Self = ExtendedTime::MIDNIGHT_24;
}
