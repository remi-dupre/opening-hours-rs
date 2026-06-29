use std::ops::Range;

use chrono::{Datelike, NaiveDate};

use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::time as ts;

use crate::Context;
use crate::localization::Localize;
use crate::utils::range::{range_intersection, ranges_union};

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
    ranges_union(time_selector.as_naive(ctx, date).filter_map(|range| {
        let range =
            range_intersection(range, ExtendedTime::MIDNIGHT_24..ExtendedTime::MIDNIGHT_48)?;

        let start = range.start.add_hours(-24)?;
        let end = range.end.add_hours(-24)?;
        Some(start..end)
    }))
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
        self.spans.iter().all(|span| span.is_immutable_full_day())
    }

    fn as_naive<'a, L: 'a + Localize>(
        &'a self,
        ctx: &'a Context<L>,
        date: NaiveDate,
    ) -> Self::Output<'a, L> {
        NaiveTimeSelectorIterator { date, ctx, inner: self.spans.iter() }
    }
}

impl TimeFilter for ts::TimeSpan {
    type Output<'a, L: 'a + Localize> = Option<Range<ExtendedTime>>;

    fn is_immutable_full_day(&self) -> bool {
        *self == Self::fixed_range(ExtendedTime::MIDNIGHT_00, ExtendedTime::MIDNIGHT_24)
    }

    fn as_naive<'a, L: 'a + Localize>(
        &'a self,
        ctx: &'a Context<L>,
        date: NaiveDate,
    ) -> Self::Output<'a, L> {
        let start_opt = self.range.start.as_naive(ctx, date);
        let end_opt = self.range.end.as_naive(ctx, date);

        let date_md = (date.month(), date.day());
        let is_summer = date_md >= (3, 20) && date_md < (9, 22);

        let (start, end) = match (start_opt, end_opt) {
            (Some(x), Some(y)) => (x, y),
            (None, Some(end)) => {
                let start = {
                    let is_morning_event = matches!(
                        self.range.start.as_time_event()?,
                        ts::TimeEvent::Sunrise | ts::TimeEvent::Dawn
                    );

                    if is_morning_event == is_summer {
                        ExtendedTime::MIDNIGHT_00
                    } else {
                        return None;
                    }
                };

                (start, end)
            }
            (Some(start), None) => {
                let end = {
                    let is_morning_event = matches!(
                        self.range.end.as_time_event()?,
                        ts::TimeEvent::Sunrise | ts::TimeEvent::Dawn
                    );

                    if is_morning_event == is_summer {
                        return None;
                    } else {
                        ExtendedTime::MIDNIGHT_24
                    }
                };

                (start, end)
            }
            (None, None) => {
                if (self.range.start <= self.range.end) == is_summer {
                    (ExtendedTime::MIDNIGHT_00, ExtendedTime::MIDNIGHT_24)
                } else {
                    return None;
                }
            }
        };

        // If end < start, it actually wraps to next day
        let end = {
            if start < end {
                end
            } else {
                end.add_hours(24)
                    .expect("overflow during TimeSpan resolution")
            }
        };

        assert!(start <= end);
        Some(start..end)
    }
}

impl TimeFilter for ts::Time {
    type Output<'a, L: 'a + Localize> = Option<ExtendedTime>;

    fn as_naive<'a, L: 'a + Localize>(
        &'a self,
        ctx: &'a Context<L>,
        date: NaiveDate,
    ) -> Self::Output<'a, L> {
        match self {
            ts::Time::Fixed(naive) => Some(*naive),
            ts::Time::Variable(variable) => variable.as_naive(ctx, date),
        }
    }
}

impl TimeFilter for ts::VariableTime {
    type Output<'a, L: 'a + Localize> = Option<ExtendedTime>;

    fn as_naive<'a, L: 'a + Localize>(
        &'a self,
        ctx: &'a Context<L>,
        date: NaiveDate,
    ) -> Self::Output<'a, L> {
        self.event.as_naive(ctx, date)?.add_minutes(self.offset)
    }
}

impl TimeFilter for ts::TimeEvent {
    type Output<'a, L: 'a + Localize> = Option<ExtendedTime>;

    fn as_naive<'a, L: 'a + Localize>(
        &'a self,
        ctx: &'a Context<L>,
        date: NaiveDate,
    ) -> Self::Output<'a, L> {
        // dbg!(date, self);
        // dbg!()
        ctx.locale.event_time(date, *self).map(Into::into)
    }
}

/// Output type for [`TimeSelector::as_naive`].
pub(crate) struct NaiveTimeSelectorIterator<'a, L: 'a + Localize> {
    date: NaiveDate,
    ctx: &'a Context<L>,
    inner: std::slice::Iter<'a, ts::TimeSpan>,
}

impl<'a, L: 'a + Localize> Iterator for NaiveTimeSelectorIterator<'a, L> {
    type Item = Range<ExtendedTime>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(span) = self.inner.next() {
            if let Some(res) = span.as_naive(self.ctx, self.date) {
                return Some(res);
            }
        }

        None
    }
}
