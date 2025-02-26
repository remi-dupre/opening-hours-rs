use std::cmp::Ordering;
use std::ops::Range;
use std::sync::Arc;

use chrono::Weekday;

use crate::normalize::paving::{Paving5D, Selector5D};
use crate::rules::day::{Month, MonthdayRange, WeekDayRange, WeekNum, WeekRange, Year, YearRange};
use crate::rules::time::{Time, TimeSpan};
use crate::sorted_vec::UniqueSortedVec;
use crate::{ExtendedTime, RuleKind};

use super::frame::{Bounded, Frame};

pub(crate) type Canonical = Paving5D<
    ExtendedTime,
    Frame<Year>,
    Frame<Month>,
    Frame<WeekNum>,
    Frame<OrderedWeekday>,
    (RuleKind, UniqueSortedVec<Arc<str>>),
>;

pub(crate) type CanonicalSelector =
    Selector5D<ExtendedTime, Frame<Year>, Frame<Month>, Frame<WeekNum>, Frame<OrderedWeekday>>;

// --
// -- OrderedWeekday
// ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OrderedWeekday(pub(crate) Weekday);

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
// -- MakeCanonical
// --

pub(crate) trait MakeCanonical: Sized + 'static {
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
