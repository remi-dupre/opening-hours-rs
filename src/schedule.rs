use std::boxed::Box;
use std::cmp::{max, min};
use std::collections::VecDeque;
use std::iter::once;
use std::ops::Range;

use crate::extended_time::ExtendedTime;
use crate::time_domain::RulesModifier;

/// Describe a full schedule for a day, keeping track of open, closed and
/// unknown periods
///
/// Internal arrays always keep a sequence of non-overlaping, increasing time
/// ranges.
#[derive(Clone, Debug, Default)]
pub struct Schedule {
    inner: Vec<(Range<ExtendedTime>, RulesModifier)>,
}

impl IntoIterator for Schedule {
    type Item = (Range<ExtendedTime>, RulesModifier);
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl Schedule {
    pub fn empty() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn from_ranges(
        ranges: impl IntoIterator<Item = Range<ExtendedTime>>,
        modifier: RulesModifier,
    ) -> Self {
        // TODO: trucate ranges to fit in day (and maybe reorder)
        Schedule {
            inner: ranges.into_iter().map(|range| (range, modifier)).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        debug_assert!(self.inner.iter().all(|(range, _)| range.start < range.end));
        self.inner.is_empty()
    }

    // NOTE: It is most likely that implementing a custom struct for this
    //       iterator could give some performances: it would avoid Boxing,
    //       Vtable, cloning inner array and allow to implement `peek()`
    //       without any wrapper.
    pub fn into_iter_filled(
        self,
    ) -> Box<dyn Iterator<Item = (Range<ExtendedTime>, RulesModifier)>> {
        let time_points = self
            .inner
            .into_iter()
            .map(|(range, modifier)| {
                let a = (range.start, modifier);
                let b = (range.end, RulesModifier::Closed);
                once(a).chain(once(b))
            })
            .flatten();

        let start = once((ExtendedTime::new(0, 0), RulesModifier::Closed));
        let end = once((ExtendedTime::new(24, 0), RulesModifier::Closed));
        let time_points = start.chain(time_points).chain(end);

        let feasibles = time_points.clone().zip(time_points.skip(1));
        let result = feasibles.filter_map(|((start, modifier), (end, _))| {
            if start < end {
                Some((start..end, modifier))
            } else {
                None
            }
        });

        Box::new(result)
    }

    // TODO: this is implemented with quadratic time where it could probably be
    //       linear.
    pub fn addition(self, mut other: Self) -> Self {
        match other.inner.pop() {
            None => self,
            Some((range, modifier)) => self.insert(range, modifier).addition(other),
        }
    }

    fn insert(self, mut ins_range: Range<ExtendedTime>, ins_modifier: RulesModifier) -> Self {
        // Build sets of intervals before and after the inserted interval

        let mut before: Vec<_> = self
            .inner
            .iter()
            .cloned()
            .filter(|(range, _)| range.start < ins_range.end)
            .filter_map(|(mut range, modifier)| {
                range.end = min(range.end, ins_range.start);

                if range.start < range.end {
                    Some((range, modifier))
                } else {
                    None
                }
            })
            .collect();

        let mut after: VecDeque<_> = self
            .inner
            .into_iter()
            .filter(|(range, _)| range.end > ins_range.start)
            .filter_map(|(mut range, modifier)| {
                range.start = max(range.start, ins_range.end);

                if range.start < range.end {
                    Some((range, modifier))
                } else {
                    None
                }
            })
            .collect();

        // Extend the inserted interval if it has adjacent intervals with same value

        while before
            .last()
            .map(|(range, modifier)| range.end == ins_range.start && *modifier == ins_modifier)
            .unwrap_or(false)
        {
            let range = before.pop().unwrap().0;
            ins_range.start = range.start;
        }

        while after
            .front()
            .map(|(range, modifier)| ins_range.end == range.start && *modifier == ins_modifier)
            .unwrap_or(false)
        {
            let range = after.pop_front().unwrap().0;
            ins_range.end = range.end;
        }

        // Build final set of intervals

        let mut inner = before;
        inner.push((ins_range, ins_modifier));
        inner.extend(after.into_iter());

        Schedule { inner }
    }
}
