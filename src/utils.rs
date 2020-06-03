use std::cmp::max;
use std::ops::Range;

use crate::extended_time::ExtendedTime;

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
