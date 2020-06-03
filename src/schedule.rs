use std::boxed::Box;
use std::iter::once;
use std::ops::Range;

use crate::extended_time::ExtendedTime;
use crate::time_domain::RulesModifier;

/// Describe a full schedule for a day, keeping track of open, closed and
/// unknown periods
///
/// Internal arrays always keep a sequence of non-overlaping, increasing time
/// ranges. The attached modifier is either Open or Closed.
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
    pub fn from_ranges(
        ranges: impl IntoIterator<Item = Range<ExtendedTime>>,
        modifier: RulesModifier,
    ) -> Self {
        let inner = {
            if modifier != RulesModifier::Closed {
                // TODO: trucate ranges to fit in day (and maybe order)
                ranges.into_iter().map(|range| (range, modifier)).collect()
            } else {
                Vec::new()
            }
        };

        Schedule { inner }
    }

    pub fn get<'a>(&'a self) -> impl Iterator<Item = (Range<ExtendedTime>, RulesModifier)> + 'a {
        self.inner.iter().cloned()
    }

    // NOTE: It is most likely that implementing a custom struct for this
    //       iterator could give some performances: it would avoid Boxing,
    //       Vtable, cloning inner array and allow to implement `peek()`
    //       without any wrapper.
    pub fn into_iter_with_closed(
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
}
