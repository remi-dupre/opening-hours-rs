use std::boxed::Box;
use std::cmp::{max, min};
use std::collections::VecDeque;
use std::fmt;
use std::iter::once;
use std::ops::Range;

use crate::extended_time::ExtendedTime;
use crate::time_domain::RuleKind;
use crate::utils::union_sorted;

#[derive(Clone)]
pub struct TimeRange {
    pub range: Range<ExtendedTime>,
    pub kind: RuleKind,
    pub comments: Vec<String>,
    _private: (),
}

impl fmt::Debug for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimeRange")
            .field("range", &self.range)
            .field("kind", &self.kind)
            .field("comments", &self.comments)
            .finish()
    }
}

impl TimeRange {
    pub fn new(range: Range<ExtendedTime>, kind: RuleKind, mut comments: Vec<String>) -> Self {
        comments.sort_unstable();
        TimeRange::new_with_sorted_comments(range, kind, comments)
    }

    pub fn new_with_sorted_comments(
        range: Range<ExtendedTime>,
        kind: RuleKind,
        comments: Vec<String>,
    ) -> Self {
        TimeRange {
            range,
            kind,
            comments,
            _private: (),
        }
    }
}

/// Describe a full schedule for a day, keeping track of open, closed and
/// unknown periods
///
/// Internal arrays always keep a sequence of non-overlaping, increasing time
/// ranges.
#[derive(Clone, Debug, Default)]
pub struct Schedule {
    inner: Vec<TimeRange>,
}

impl IntoIterator for Schedule {
    type Item = TimeRange;
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
        kind: RuleKind,
        mut comments: Vec<String>,
    ) -> Self {
        comments.sort_unstable();

        Schedule {
            inner: ranges
                .into_iter()
                .inspect(|range| assert!(range.start < range.end))
                .map(|range| TimeRange::new_with_sorted_comments(range, kind, comments.clone()))
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    // NOTE: It is most likely that implementing a custom struct for this
    //       iterator could give some performances: it would avoid Boxing,
    //       Vtable, cloning inner array and allow to implement `peek()`
    //       without any wrapper.
    pub fn into_iter_filled(self) -> Box<dyn Iterator<Item = TimeRange>> {
        let time_points = self
            .inner
            .into_iter()
            .map(|tr| {
                let a = (tr.range.start, tr.kind, tr.comments);
                let b = (tr.range.end, RuleKind::Closed, Vec::new());
                once(a).chain(once(b))
            })
            .flatten();

        let start = once((ExtendedTime::new(0, 0), RuleKind::Closed, Vec::new()));
        let end = once((ExtendedTime::new(24, 0), RuleKind::Closed, Vec::new()));
        let time_points = start.chain(time_points).chain(end);

        let feasibles = time_points.clone().zip(time_points.skip(1));
        let result = feasibles.filter_map(|((start, kind, comments), (end, _, _))| {
            if start < end {
                Some(TimeRange::new_with_sorted_comments(
                    start..end,
                    kind,
                    comments,
                ))
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
            Some(tr) => self.insert(tr).addition(other),
        }
    }

    fn insert(self, mut ins_tr: TimeRange) -> Self {
        // Build sets of intervals before and after the inserted interval

        let ins_start = ins_tr.range.start;
        let ins_end = ins_tr.range.end;

        let mut before: Vec<_> = self
            .inner
            .iter()
            .cloned()
            .filter(|tr| tr.range.start < ins_end)
            .filter_map(|mut tr| {
                tr.range.end = min(tr.range.end, ins_tr.range.start);

                if tr.range.start < tr.range.end {
                    Some(tr)
                } else {
                    ins_tr.comments = union_sorted(&ins_tr.comments, &tr.comments);
                    None
                }
            })
            .collect();

        let mut after: VecDeque<_> = self
            .inner
            .into_iter()
            .filter(|tr| tr.range.end > ins_start)
            .filter_map(|mut tr| {
                tr.range.start = max(tr.range.start, ins_tr.range.end);

                if tr.range.start < tr.range.end {
                    Some(tr)
                } else {
                    ins_tr.comments = union_sorted(&ins_tr.comments, &tr.comments);
                    None
                }
            })
            .collect();

        // Extend the inserted interval if it has adjacent intervals with same value

        while before
            .last()
            .map(|tr| tr.range.end == ins_tr.range.start && tr.kind == ins_tr.kind)
            .unwrap_or(false)
        {
            let tr = before.pop().unwrap();
            ins_tr.range.start = tr.range.start;
            ins_tr.comments = union_sorted(&tr.comments, &ins_tr.comments);
        }

        while after
            .front()
            .map(|tr| ins_tr.range.end == tr.range.start && tr.kind == ins_tr.kind)
            .unwrap_or(false)
        {
            let tr = after.pop_front().unwrap();
            ins_tr.range.end = tr.range.end;
            ins_tr.comments = union_sorted(&tr.comments, &ins_tr.comments);
        }

        // Build final set of intervals

        let mut inner = before;
        inner.push(ins_tr);
        inner.extend(after.into_iter());

        Schedule { inner }
    }
}
