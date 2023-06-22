use std::ops::Range;

use chrono::NaiveDate;

use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::time as ts;

use crate::context::Context;
use crate::localize::Localize;
use crate::utils::range::{range_intersection, ranges_union};

pub(crate) fn time_selector_intervals_at<'c, L: Localize>(
    ctx: &'c Context<L>,
    time_selector: &'c ts::TimeSelector,
    date: NaiveDate,
) -> impl Iterator<Item = Range<ExtendedTime>> + 'c {
    ranges_union(
        time_selector_as_naive(ctx, time_selector, date).filter_map(|range| {
            let dstart = ExtendedTime::new(0, 0);
            let dend = ExtendedTime::new(24, 0);
            range_intersection(range, dstart..dend)
        }),
    )
}

pub(crate) fn time_selector_intervals_at_next_day<'c, L: Localize>(
    ctx: &'c Context<L>,
    time_selector: &'c ts::TimeSelector,
    date: NaiveDate,
) -> impl Iterator<Item = Range<ExtendedTime>> + 'c {
    ranges_union(
        time_selector_as_naive(ctx, time_selector, date)
            .filter_map(|range| {
                let dstart = ExtendedTime::new(24, 0);
                let dend = ExtendedTime::new(48, 0);
                range_intersection(range, dstart..dend)
            })
            .map(|range| {
                let start = range.start.add_hours(-24).unwrap();
                let end = range.end.add_hours(-24).unwrap();
                start..end
            }),
    )
}

fn time_selector_as_naive<'c, L: Localize>(
    ctx: &'c Context<L>,
    time_selector: &'c ts::TimeSelector,
    date: NaiveDate,
) -> impl Iterator<Item = Range<ExtendedTime>> + 'c {
    time_selector
        .time
        .iter()
        .map(move |span| span.as_naive(ctx, date))
}

/// Trait used to project a time representation to its naive representation at
/// a given date.
pub(crate) trait AsNaive {
    type Output;
    fn as_naive<L: Localize>(&self, ctx: &Context<L>, date: NaiveDate) -> Self::Output;
}

impl AsNaive for ts::TimeSpan {
    type Output = Range<ExtendedTime>;

    fn as_naive<L: Localize>(&self, ctx: &Context<L>, date: NaiveDate) -> Self::Output {
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

impl AsNaive for ts::Time {
    type Output = ExtendedTime;

    fn as_naive<L: Localize>(&self, ctx: &Context<L>, date: NaiveDate) -> Self::Output {
        match self {
            ts::Time::Fixed(naive) => *naive,
            ts::Time::Variable(variable) => variable.as_naive(ctx, date),
        }
    }
}

impl AsNaive for ts::VariableTime {
    type Output = ExtendedTime;

    fn as_naive<L: Localize>(&self, ctx: &Context<L>, date: NaiveDate) -> Self::Output {
        self.event
            .as_naive(ctx, date)
            .add_minutes(self.offset)
            .unwrap_or_else(|_| ExtendedTime::new(0, 0))
    }
}

impl AsNaive for ts::TimeEvent {
    type Output = ExtendedTime;

    fn as_naive<L: Localize>(&self, ctx: &Context<L>, _date: NaiveDate) -> Self::Output {
        // TODO: real computation based on the day (and position/timezone?)
        match self {
            Self::Dawn => ExtendedTime::new(6, 0),
            Self::Sunrise => ExtendedTime::new(7, 0),
            Self::Sunset => ExtendedTime::new(19, 0),
            Self::Dusk => ExtendedTime::new(20, 0),
        }
    }
}
