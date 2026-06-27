use alloc::vec::Vec;
use core::ops::Range;

use crate::normalize::bounded::{Bounded, Frame, UpperBounded};
use crate::normalize::canonical_time::TimeRules;
use crate::normalize::paving::{EmptyPavingSelector, Paving4D, Selector4D};
use crate::rules::day::{
    DaySelector, Month, MonthdayRange, WeekDayRange, WeekNum, WeekRange, Year, YearRange,
};
use crate::util::weekday::OrderedWeekday;

/// A canonical day representation, built from simple selector intervals of year, month, weekday and
/// days. It maps to a TimeRules that may or may not have been normalized yet.
pub(crate) type CanonicalDate =
    Paving4D<Frame<OrderedWeekday>, WeekNum, Frame<Month>, Year, TimeRules>;

// --
// -- CanonicalDateSelector
// --

/// The selector type that can operator on CanonicalDate.
/// It can be converted back and forth from a DaySelector.
pub(crate) type CanonicalDateSelector =
    Selector4D<Frame<OrderedWeekday>, WeekNum, Frame<Month>, Year>;

impl TryFrom<&DaySelector> for CanonicalDateSelector {
    type Error = ();

    fn try_from(value: &DaySelector) -> Result<Self, Self::Error> {
        let selector = EmptyPavingSelector
            .dim_front(MakeCanonical::try_from_iterator(&value.year).ok_or(())?)
            .dim_front(MakeCanonical::try_from_iterator(&value.monthday).ok_or(())?)
            .dim_front(MakeCanonical::try_from_iterator(&value.week).ok_or(())?)
            .dim_front(MakeCanonical::try_from_iterator(&value.weekday).ok_or(())?);

        Ok(selector)
    }
}

impl From<CanonicalDateSelector> for DaySelector {
    fn from(val: CanonicalDateSelector) -> Self {
        let (rgs_weekday, val) = val.into_unpack_front();
        let (rgs_week, val) = val.into_unpack_front();
        let (rgs_monthday, val) = val.into_unpack_front();
        let (rgs_year, EmptyPavingSelector) = val.into_unpack_front();

        DaySelector {
            year: MakeCanonical::into_selector(rgs_year, true),
            monthday: MakeCanonical::into_selector(rgs_monthday, true),
            week: MakeCanonical::into_selector(rgs_week, true),
            weekday: MakeCanonical::into_selector(rgs_weekday, true),
        }
    }
}

// --
// -- MakeCanonical
// --

/// Small trait to help convert from and into a selector.
trait MakeCanonical: Sized + 'static {
    type CanonicalType: UpperBounded;
    fn try_make_canonical(&self) -> Option<Range<Self::CanonicalType>>;
    fn into_type(canonical: Range<Self::CanonicalType>) -> Option<Self>;

    fn try_from_iterator<'a>(
        iter: impl IntoIterator<Item = &'a Self>,
    ) -> Option<Vec<Range<Self::CanonicalType>>> {
        let mut ranges = Vec::new();

        for elem in iter {
            let range = Self::try_make_canonical(elem)?;
            ranges.extend(UpperBounded::split_inverted_range(range));
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
    type CanonicalType = Year;

    fn try_make_canonical(&self) -> Option<Range<Self::CanonicalType>> {
        let (range, step) = self.into_parts();

        if step != 1 {
            return None;
        }

        Some(*range.start()..range.end().succ()?)
    }

    fn into_type(canonical: Range<Self::CanonicalType>) -> Option<Self> {
        YearRange::new(canonical.start..=canonical.end.pred()?, 1)
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
    type CanonicalType = WeekNum;

    fn try_make_canonical(&self) -> Option<Range<Self::CanonicalType>> {
        let (range, step) = self.into_parts();

        if step != 1 {
            return None;
        }

        Some(*range.start()..range.end().succ()?)
    }

    fn into_type(canonical: Range<Self::CanonicalType>) -> Option<Self> {
        WeekRange::new(canonical.start..=canonical.end.pred()?, 1)
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
