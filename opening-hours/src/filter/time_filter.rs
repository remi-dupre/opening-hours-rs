use std::ops::Range;

use chrono::{Duration, NaiveDate};

use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::time as ts;

use crate::localization::Localize;
use crate::utils::range::{range_intersection, ranges_union};
use crate::Context;

pub(crate) fn time_selector_intervals_at<'a, L: 'a + Localize>(
    ctx: &'a Context<L>,
    time_selector: &'a ts::TimeSelector,
    date: NaiveDate,
) -> impl Iterator<Item = Range<ExtendedTime>> + 'a {
    ranges_union(time_selector.as_naive(ctx, date).filter_map(|range| {
        range_intersection(range, ExtendedTime::MIDNIGHT_00..ExtendedTime::MIDNIGHT_24)
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
                range_intersection(range, ExtendedTime::MIDNIGHT_24..ExtendedTime::MIDNIGHT_48)
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
        NaiveTimeSelectorIterator {
            date,
            ctx,
            spans: self.time.iter(),
            span_iter: None,
        }
    }
}

impl TimeFilter for ts::TimeSpan {
    type Output<'a, L: 'a + Localize> = NaiveTimeSpanIterator;

    fn is_immutable_full_day(&self) -> bool {
        *self == Self::fixed_range(ExtendedTime::MIDNIGHT_00, ExtendedTime::MIDNIGHT_24)
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

        NaiveTimeSpanIterator { time: Some(start), end, repeats: self.repeats }
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
            .unwrap_or(ExtendedTime::MIDNIGHT_00)
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

pub(crate) struct NaiveTimeSpanIterator {
    time: Option<ExtendedTime>,
    end: ExtendedTime,
    repeats: Option<Duration>,
}

impl Iterator for NaiveTimeSpanIterator {
    type Item = Range<ExtendedTime>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(repeats) = self.repeats {
            let time = self.time?;

            let next_time = time
                .add_minutes(
                    repeats
                        .num_minutes()
                        .try_into()
                        .expect("repeats duration too long"),
                )
                .expect("overflow during TimeSpan repetition");
            if next_time <= self.end {
                self.time = Some(next_time);
            } else {
                self.time = None;
            }

            Some(time..time)
        } else {
            Some(self.time.take()?..self.end)
        }
    }
}

/// Output type for [`TimeSelector::as_naive`].
pub(crate) struct NaiveTimeSelectorIterator<'a, L: 'a + Localize> {
    date: NaiveDate,
    ctx: &'a Context<L>,
    spans: std::slice::Iter<'a, ts::TimeSpan>,
    span_iter: Option<NaiveTimeSpanIterator>,
}

impl<'a, L: 'a + Localize> Iterator for NaiveTimeSelectorIterator<'a, L> {
    type Item = Range<ExtendedTime>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(range) = self.span_iter.as_mut().and_then(|iter| iter.next()) {
                return Some(range);
            }
            self.span_iter = Some(self.spans.next()?.as_naive(self.ctx, self.date));
        }
    }
}
