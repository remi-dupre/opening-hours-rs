use std::ops::Range;

use chrono::NaiveDate;

use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::time as ts;

use crate::utils::range::{range_intersection, ranges_union};

pub(crate) fn time_selector_intervals_at(
    time_selector: &ts::TimeSelector,
    date: NaiveDate,
) -> impl Iterator<Item = Range<ExtendedTime>> + '_ {
    ranges_union(time_selector.as_naive(date).filter_map(|range| {
        let dstart = ExtendedTime::new(0, 0).unwrap();
        let dend = ExtendedTime::new(24, 0).unwrap();
        range_intersection(range, dstart..dend)
    }))
}

pub(crate) fn time_selector_intervals_at_next_day(
    time_selector: &ts::TimeSelector,
    date: NaiveDate,
) -> impl Iterator<Item = Range<ExtendedTime>> + '_ {
    ranges_union(
        time_selector
            .as_naive(date)
            .filter_map(|range| {
                let dstart = ExtendedTime::new(24, 0).unwrap();
                let dend = ExtendedTime::new(48, 0).unwrap();
                range_intersection(range, dstart..dend)
            })
            .map(|range| {
                let start = range.start.add_hours(-24).unwrap();
                let end = range.end.add_hours(-24).unwrap();
                start..end
            }),
    )
}

/// Trait used to project a time representation to its naive representation at
/// a given date.
pub(crate) trait TimeFilter {
    type Output<'a>
    where
        Self: 'a;

    /// Check if this filter is always matching the full day.
    fn is_immutable_full_day(&self) -> bool {
        false
    }

    /// Project a time representation to its naive representation at a given date.
    fn as_naive(&self, date: NaiveDate) -> Self::Output<'_>;
}

impl TimeFilter for ts::TimeSelector {
    type Output<'a> = NaiveTimeSelectorIterator<'a>;

    fn is_immutable_full_day(&self) -> bool {
        self.time.iter().all(|span| span.is_immutable_full_day())
    }

    fn as_naive(&self, date: NaiveDate) -> Self::Output<'_> {
        NaiveTimeSelectorIterator { date, inner: self.time.iter() }
    }
}

impl TimeFilter for ts::TimeSpan {
    type Output<'a> = Range<ExtendedTime>;

    fn is_immutable_full_day(&self) -> bool {
        self.range.start == ts::Time::Fixed(ExtendedTime::new(0, 0).unwrap())
            && self.range.end == ts::Time::Fixed(ExtendedTime::new(24, 0).unwrap())
    }

    fn as_naive(&self, date: NaiveDate) -> Self::Output<'_> {
        let start = self.range.start.as_naive(date);
        let end = self.range.end.as_naive(date);

        // If end < start, it actually wraps to next day
        let end = {
            if start <= end {
                end
            } else {
                end.add_hours(24)
                    .expect("overflow during TimeSpan resolution")
            }
        };

        assert!(start <= end);
        start..end
    }
}

impl TimeFilter for ts::Time {
    type Output<'a> = ExtendedTime;

    fn as_naive(&self, date: NaiveDate) -> Self::Output<'_> {
        match self {
            ts::Time::Fixed(naive) => *naive,
            ts::Time::Variable(variable) => variable.as_naive(date),
        }
    }
}

impl TimeFilter for ts::VariableTime {
    type Output<'a> = ExtendedTime;

    fn as_naive(&self, date: NaiveDate) -> Self::Output<'_> {
        self.event
            .as_naive(date)
            .add_minutes(self.offset)
            .unwrap_or_else(|| ExtendedTime::new(0, 0).unwrap())
    }
}

impl TimeFilter for ts::TimeEvent {
    type Output<'a> = ExtendedTime;

    fn as_naive(&self, _date: NaiveDate) -> Self::Output<'_> {
        // TODO: real computation based on the day (and position/timezone?)
        match self {
            Self::Dawn => ExtendedTime::new(6, 0).unwrap(),
            Self::Sunrise => ExtendedTime::new(7, 0).unwrap(),
            Self::Sunset => ExtendedTime::new(19, 0).unwrap(),
            Self::Dusk => ExtendedTime::new(20, 0).unwrap(),
        }
    }
}

/// Output type for [`TimeSelector::as_naive`].
pub(crate) struct NaiveTimeSelectorIterator<'a> {
    date: NaiveDate,
    inner: std::slice::Iter<'a, ts::TimeSpan>,
}

impl<'a> Iterator for NaiveTimeSelectorIterator<'a> {
    type Item = <ts::TimeSpan as TimeFilter>::Output<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let span = self.inner.next()?;
        Some(span.as_naive(self.date))
    }
}
