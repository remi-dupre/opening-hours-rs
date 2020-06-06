use std::cmp::{max, min, Ordering};
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

pub fn is_sorted<T: PartialOrd>(slice: &[T]) -> bool {
    if let Some(mut curr) = slice.first() {
        for x in &slice[1..] {
            if x >= curr {
                curr = x;
            } else {
                return false;
            }
        }
    }

    true
}

pub fn union_sorted<T: Clone + Ord>(vec_1: &[T], vec_2: &[T]) -> Vec<T> {
    debug_assert!(is_sorted(vec_1));
    debug_assert!(is_sorted(vec_2));

    match (vec_1, vec_2) {
        ([], vec) | (vec, []) => vec.to_vec(),
        ([head_1 @ .., tail_1], [head_2 @ .., tail_2]) => {
            let build_with = |for_head_1, for_head_2, tail| {
                let mut res = union_sorted(for_head_1, for_head_2);
                res.push(tail);
                res
            };

            match tail_1.cmp(tail_2) {
                Ordering::Equal => build_with(head_1, head_2, tail_1.clone()),
                Ordering::Less => build_with(vec_1, head_2, tail_2.clone()),
                Ordering::Greater => build_with(head_1, vec_2, tail_1.clone()),
            }
        }
    }
}
