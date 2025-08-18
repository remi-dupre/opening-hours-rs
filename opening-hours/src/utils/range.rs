use std::cmp::{max, min};
use std::ops::Range;
use std::sync::Arc;

use chrono::NaiveDateTime;
use opening_hours_syntax::rules::RuleKind;

// DateTimeRange

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DateTimeRange<D = NaiveDateTime> {
    pub range: Range<D>,
    pub kind: RuleKind,
    pub comment: Arc<str>,
}

impl<D> DateTimeRange<D> {
    /// Extract the kind and comment from the range, which are the values that define current state
    /// of an expression.
    pub fn into_state(self) -> (RuleKind, Arc<str>) {
        (self.kind, self.comment)
    }
}

// Range operations

pub(crate) fn ranges_union<T: Ord>(
    ranges: impl IntoIterator<Item = Range<T>>,
) -> impl Iterator<Item = Range<T>> {
    // TODO (optimisation): we could gain performance by ensuring that range iterators are always
    // sorted.
    let mut ranges: Vec<_> = ranges.into_iter().collect();
    ranges.sort_unstable_by(|r1, r2| r1.start.cmp(&r2.start));

    // Get ranges by increasing start
    let mut ranges = ranges.into_iter();
    let mut current_opt = ranges.next();

    std::iter::from_fn(move || {
        if let Some(ref mut current) = current_opt {
            #[allow(clippy::while_let_on_iterator)]
            while let Some(item) = ranges.next() {
                if current.end >= item.start {
                    // The two intervals intersect with each other
                    if item.end > current.end {
                        current.end = item.end;
                    }
                } else {
                    return Some(current_opt.replace(item).unwrap());
                }
            }

            Some(current_opt.take().unwrap())
        } else {
            None
        }
    })
}

pub(crate) fn range_intersection<T: Ord>(range_1: Range<T>, range_2: Range<T>) -> Option<Range<T>> {
    let result = max(range_1.start, range_2.start)..min(range_1.end, range_2.end);

    if result.start < result.end {
        Some(result)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::{range_intersection, ranges_union};

    #[test]
    fn test_unions() {
        assert_eq!(
            &ranges_union([1..5, 0..1, 3..7, 8..9]).collect::<Vec<_>>(),
            &[0..7, 8..9]
        );
    }

    #[test]
    fn test_intersection() {
        assert!(range_intersection(0..1, 1..2).is_none());
        assert_eq!(range_intersection(0..3, 1..2).unwrap(), 1..2);
        assert_eq!(range_intersection(0..3, 2..4).unwrap(), 2..3);
    }
}
