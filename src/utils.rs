use std::cmp::{max, min, Ordering};
use std::fmt;
use std::ops::{Range, RangeInclusive};

use chrono::NaiveDateTime;
use opening_hours_syntax::rules::RuleKind;

// DateTimeRange

#[non_exhaustive]
#[derive(Clone, Eq, PartialEq)]
pub struct DateTimeRange<'c> {
    pub range: Range<NaiveDateTime>,
    pub kind: RuleKind,
    pub(crate) comments: Vec<&'c str>,
}

impl fmt::Debug for DateTimeRange<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DateTimeRange")
            .field("range", &self.range)
            .field("kind", &self.kind)
            .field("comments", &self.comments)
            .finish()
    }
}

impl<'c> DateTimeRange<'c> {
    pub(crate) fn new_with_sorted_comments(
        range: Range<NaiveDateTime>,
        kind: RuleKind,
        comments: Vec<&'c str>,
    ) -> Self {
        Self {
            range,
            kind,
            comments,
        }
    }

    pub fn comments(&self) -> &[&'c str] {
        &self.comments
    }

    pub fn into_comments(self) -> Vec<&'c str> {
        self.comments
    }
}

// Range operations

pub(crate) fn wrapping_range_contains<T: PartialOrd>(range: &RangeInclusive<T>, elt: &T) -> bool {
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

pub(crate) fn time_ranges_union<T: Ord>(
    ranges: impl Iterator<Item = Range<T>>,
) -> impl Iterator<Item = Range<T>> {
    // TODO: we could gain performance by ensuring that range iterators are
    //       always sorted.
    let mut ranges: Vec<_> = ranges.collect();
    ranges.sort_unstable_by(|r1, r2| r1.start.cmp(&r2.start));

    // Get ranges by increasing start
    let mut ranges = ranges.into_iter();
    let mut current_opt = ranges.next();

    std::iter::from_fn(move || {
        if let Some(ref mut current) = current_opt {
            #[allow(clippy::clippy::while_let_on_iterator)]
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

pub(crate) fn union_sorted<T: Clone + Ord>(vec_1: &[T], vec_2: &[T]) -> Vec<T> {
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
