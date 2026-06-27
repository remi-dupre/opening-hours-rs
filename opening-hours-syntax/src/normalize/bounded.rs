//! Helpers to convert from open-ended ranges from and to close-ended ranges.

// --
// -- Framable
// --

use core::cmp::Ordering;
use core::ops::{Range, RangeInclusive};

use chrono::Weekday;

use crate::ExtendedTime;
use crate::rules::day::{Month, WeekNum, Year};
use crate::util::weekday::OrderedWeekday;

// --
// -- Bounded
// --

/// A type with an upper and a lower bound.
pub(crate) trait Bounded: Sized + PartialOrd + Ord {
    const BOUND_START: Self;
    const BOUND_END: Self;

    fn succ(self) -> Option<Self>;
    fn pred(self) -> Option<Self>;
}

impl Bounded for OrderedWeekday {
    const BOUND_START: Self = OrderedWeekday(Weekday::Mon);
    const BOUND_END: Self = OrderedWeekday(Weekday::Sun);

    fn succ(self) -> Option<Self> {
        if self == Self::BOUND_END {
            None
        } else {
            Some(OrderedWeekday(self.0.succ()))
        }
    }

    fn pred(self) -> Option<Self> {
        if self == Self::BOUND_START {
            None
        } else {
            Some(OrderedWeekday(self.0.pred()))
        }
    }
}

impl Bounded for Month {
    const BOUND_START: Self = Month::January;
    const BOUND_END: Self = Month::December;

    fn succ(self) -> Option<Self> {
        if self == Self::BOUND_END {
            None
        } else {
            Some(self.next())
        }
    }

    fn pred(self) -> Option<Self> {
        if self == Self::BOUND_START {
            None
        } else {
            Some(self.prev())
        }
    }
}

impl Bounded for Year {
    const BOUND_START: Self = Year(1900);
    const BOUND_END: Self = Year(9999);

    fn succ(self) -> Option<Self> {
        if self < Self::BOUND_END {
            Some(Year(self.0 + 1))
        } else {
            None
        }
    }

    fn pred(self) -> Option<Self> {
        if self > Self::BOUND_START {
            Some(Year(self.0 - 1))
        } else {
            None
        }
    }
}

impl Bounded for WeekNum {
    const BOUND_START: Self = WeekNum(1);
    const BOUND_END: Self = WeekNum(53);

    fn succ(self) -> Option<Self> {
        if self < Self::BOUND_END {
            Some(WeekNum(*self % 53 + 1))
        } else {
            None
        }
    }

    fn pred(self) -> Option<Self> {
        if self > Self::BOUND_START {
            Some(WeekNum((*self + 51) % 53 + 1))
        } else {
            None
        }
    }
}

impl Bounded for ExtendedTime {
    const BOUND_START: Self = Self::MIDNIGHT_00;
    const BOUND_END: Self = Self::new(47, 59).unwrap();

    fn succ(self) -> Option<Self> {
        if self < Self::BOUND_END {
            self.add_minutes(1)
        } else {
            None
        }
    }

    fn pred(self) -> Option<Self> {
        self.add_minutes(-1)
    }
}

// --
// -- UpperBounded
// --

/// A type with a lower bound and an open ended upper bound.
pub(crate) trait UpperBounded: Bounded {
    /// An unreachable upper bound
    const BOUND_UPPER: Self; // Excluded

    fn bounds() -> Range<Self> {
        Self::BOUND_START..Self::BOUND_UPPER
    }

    // Ensure that input range is "increasing", otherwise it is splited into two ranges:
    // [bounds.start, range.end[ and [range.start, bounds.end[
    fn split_inverted_range(range: Range<Self>) -> impl Iterator<Item = Range<Self>> {
        if range.start >= range.end {
            // start == end when a wrapping range gets expanded from exclusive to inclusive range
            core::iter::once(Self::BOUND_START..range.end)
                .chain(Some(range.start..Self::BOUND_UPPER))
        } else {
            core::iter::once(range).chain(None)
        }
    }

    fn to_range_strict(range: RangeInclusive<Self>) -> Range<Self> {
        let (start, end) = range.into_inner();
        let strict_end = end.succ().unwrap_or(Self::BOUND_UPPER);
        start..strict_end
    }

    fn to_range_inclusive(range: Range<Self>) -> Option<RangeInclusive<Self>> {
        if range.end <= range.start {
            None
        } else if range.end == Self::BOUND_UPPER {
            Some(range.start..=Self::BOUND_END)
        } else {
            Some(range.start..=range.end.pred()?)
        }
    }
}

impl UpperBounded for ExtendedTime {
    const BOUND_UPPER: Self = ExtendedTime::MIDNIGHT_48;
}

impl UpperBounded for Year {
    const BOUND_UPPER: Year = Year(10_000);
}

impl UpperBounded for WeekNum {
    const BOUND_UPPER: WeekNum = WeekNum(54);
}

// --
// -- Frame
// --

/// Allows to wrap a Bounded type to implement UpperBounded if not already available.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Frame<T: Bounded> {
    Val(T),
    End,
}

impl<T: Bounded> From<T> for Frame<T> {
    fn from(value: T) -> Self {
        Self::Val(value)
    }
}

impl<T: Bounded> Frame<T> {
    pub(crate) fn into_val(self) -> Option<T> {
        match self {
            Self::Val(x) => Some(x),
            Self::End => None,
        }
    }
}

impl<T: Bounded> Ord for Frame<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Frame::Val(x), Frame::Val(y)) => x.cmp(y),
            (Frame::Val(_), Frame::End) => Ordering::Less,
            (Frame::End, Frame::Val(_)) => Ordering::Greater,
            (Frame::End, Frame::End) => Ordering::Equal,
        }
    }
}

impl<T: Bounded> PartialOrd for Frame<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Bounded> Bounded for Frame<T> {
    const BOUND_START: Self = Frame::Val(T::BOUND_START);
    const BOUND_END: Self = Frame::Val(T::BOUND_END);

    fn succ(self) -> Option<Self> {
        match self {
            Frame::Val(x) => x.succ().map(Frame::Val),
            Frame::End => None,
        }
    }

    fn pred(self) -> Option<Self> {
        match self {
            Frame::Val(x) => x.pred().map(Frame::Val),
            Frame::End => None,
        }
    }
}

impl<T: Bounded> UpperBounded for Frame<T> {
    const BOUND_UPPER: Self = Frame::End;
}
