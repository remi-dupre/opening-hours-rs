use std::cmp::Ordering;
use std::ops::RangeInclusive;

pub fn wrapping_range_contains<T: PartialOrd>(range: &RangeInclusive<T>, elt: &T) -> bool {
    if range.start() <= range.end() {
        range.contains(elt)
    } else {
        range.start() <= elt || elt <= range.end()
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
