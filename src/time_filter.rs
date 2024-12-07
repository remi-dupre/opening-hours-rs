use std::ops::Range;

use chrono::NaiveDate;

use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::time as ts;

use crate::utils::range::{range_intersection, ranges_union};
use crate::{Context, Localize};

pub(crate) fn time_selector_intervals_at<'a, L: 'a + Localize>(
    ctx: &'a Context<L>,
    time_selector: &'a ts::TimeSelector,
    date: NaiveDate,
) -> impl Iterator<Item = Range<ExtendedTime>> + 'a {
    ranges_union(time_selector.as_naive(ctx, date).filter_map(|range| {
        let dstart = ExtendedTime::new(0, 0).unwrap();
        let dend = ExtendedTime::new(24, 0).unwrap();
        range_intersection(range, dstart..dend)
    }))
}

pub(crate) fn time_selector_intervals_at_next_day<'a, L: 'a + Localize>(
    ctx: &'a Context<L>,
    time_selector: &'a ts::TimeSelector,
    date: NaiveDate,
) -> impl Iterator<Item = Range<ExtendedTime>> + 'a {
    ranges_union(
        time_selector
            .as_naive(ctx, date)
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
    type Output<'a, L: 'a + Localize>
    where
        Self: 'a;

    /// Check if this filter is always matching the full day.
    fn is_immutable_full_day(&self) -> bool {
        false
    }

    /// Project a time representation to its naive representation at a given date.
    fn as_naive<'a, L: 'a + Localize>(
        &'a self,
        ctx: &'a Context<L>,
        date: NaiveDate,
    ) -> Self::Output<'a, L>;
}

impl TimeFilter for ts::TimeSelector {
    type Output<'a, L: 'a + Localize> = NaiveTimeSelectorIterator<'a, L>;

    fn is_immutable_full_day(&self) -> bool {
        self.time.iter().all(|span| span.is_immutable_full_day())
    }

    fn as_naive<'a, L: 'a + Localize>(
        &'a self,
        ctx: &'a Context<L>,
        date: NaiveDate,
    ) -> Self::Output<'a, L> {
        NaiveTimeSelectorIterator { date, ctx, inner: self.time.iter() }
    }
}

impl TimeFilter for ts::TimeSpan {
    type Output<'a, L: 'a + Localize> = Range<ExtendedTime>;

    fn is_immutable_full_day(&self) -> bool {
        self.range.start == ts::Time::Fixed(ExtendedTime::new(0, 0).unwrap())
            && self.range.end == ts::Time::Fixed(ExtendedTime::new(24, 0).unwrap())
    }

    fn as_naive<'a, L: 'a + Localize>(
        &'a self,
        ctx: &'a Context<L>,
        date: NaiveDate,
    ) -> Self::Output<'a, L> {
        let start = self.range.start.as_naive(ctx, date);
        let end = self.range.end.as_naive(ctx, date);

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
    type Output<'a, L: 'a + Localize> = ExtendedTime;

    fn as_naive<'a, L: 'a + Localize>(
        &'a self,
        ctx: &'a Context<L>,
        date: NaiveDate,
    ) -> Self::Output<'a, L> {
        match self {
            ts::Time::Fixed(naive) => *naive,
            ts::Time::Variable(variable) => variable.as_naive(ctx, date),
        }
    }
}

impl TimeFilter for ts::VariableTime {
    type Output<'a, L: 'a + Localize> = ExtendedTime;

    fn as_naive<'a, L: 'a + Localize>(
        &'a self,
        ctx: &'a Context<L>,
        date: NaiveDate,
    ) -> Self::Output<'a, L> {
        self.event
            .as_naive(ctx, date)
            .add_minutes(self.offset)
            .unwrap_or_else(|| ExtendedTime::new(0, 0).unwrap())
    }
}

impl TimeFilter for ts::TimeEvent {
    type Output<'a, L: 'a + Localize> = ExtendedTime;

    fn as_naive<'a, L: 'a + Localize>(
        &'a self,
        ctx: &'a Context<L>,
        date: NaiveDate,
    ) -> Self::Output<'a, L> {
        ctx.locale.event_time(date, *self).into()
    }
}

/// Output type for [`TimeSelector::as_naive`].
pub(crate) struct NaiveTimeSelectorIterator<'a, L: 'a + Localize> {
    date: NaiveDate,
    ctx: &'a Context<L>,
    inner: std::slice::Iter<'a, ts::TimeSpan>,
}

impl<'a, L: 'a + Localize> Iterator for NaiveTimeSelectorIterator<'a, L> {
    type Item = <ts::TimeSpan as TimeFilter>::Output<'a, L>;

    fn next(&mut self) -> Option<Self::Item> {
        let span = self.inner.next()?;
        Some(span.as_naive(self.ctx, self.date))
    }
}
