use std::cmp::{max, min};
use std::ops::Range;

use crate::extended_time::ExtendedTime;

// TODO: can we gain performances by returning an iterator (requires custom implem).
pub fn time_ranges_union(
    ranges: impl Iterator<Item = Range<ExtendedTime>>,
) -> Vec<Range<ExtendedTime>> {
    let mut ranges: Vec<_> = ranges.collect();
    let mut output = Vec::new();

    // Get ranges by increasing start
    ranges.sort_unstable_by_key(|range| range.start);
    let mut ranges = ranges.into_iter();

    if let Some(mut current) = ranges.next() {
        for item in ranges {
            assert!(item.start >= current.start);

            if current.end >= item.start {
                // The two intervals intersect with each other
                current.end = max(current.end, item.end);
            } else {
                output.push(current);
                current = item;
            }
        }

        output.push(current);
    }

    output
}

pub fn range_intersection<T: Ord>(range_1: Range<T>, range_2: Range<T>) -> Option<Range<T>> {
    let result = max(range_1.start, range_2.start)..min(range_1.end, range_2.end);

    if result.start < result.end {
        Some(result)
    } else {
        None
    }
}
