use std::boxed::Box;
use std::cmp::{max, min};
use std::fmt;
use std::iter::once;
use std::mem::take;
use std::ops::Range;

use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::RuleKind;
use opening_hours_syntax::sorted_vec::UniqueSortedVec;

#[non_exhaustive]
#[derive(Clone, Eq, PartialEq)]
pub struct TimeRange<'c> {
    pub range: Range<ExtendedTime>,
    pub kind: RuleKind,
    pub comments: UniqueSortedVec<&'c str>,
}

impl<'c> TimeRange<'c> {
    pub(crate) fn new(
        range: Range<ExtendedTime>,
        kind: RuleKind,
        comments: UniqueSortedVec<&'c str>,
    ) -> Self {
        TimeRange {
            range,
            kind,
            comments,
        }
    }
}

impl fmt::Debug for TimeRange<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimeRange")
            .field("range", &self.range)
            .field("kind", &self.kind)
            .field("comments", &self.comments)
            .finish()
    }
}

/// Describe a full schedule for a day, keeping track of open, closed and
/// unknown periods
///
/// Internal arrays always keep a sequence of non-overlaping, increasing time
/// ranges.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Schedule<'c> {
    pub(crate) inner: Vec<TimeRange<'c>>,
}

impl<'c> IntoIterator for Schedule<'c> {
    type Item = TimeRange<'c>;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'c> Schedule<'c> {
    pub fn empty() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn from_ranges(
        ranges: impl IntoIterator<Item = Range<ExtendedTime>>,
        kind: RuleKind,
        comments: UniqueSortedVec<&'c str>,
    ) -> Self {
        Schedule {
            inner: ranges
                .into_iter()
                .inspect(|range| assert!(range.start < range.end))
                .map(|range| TimeRange::new(range, kind, comments.clone()))
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
    pub fn into_iter_filled(self) -> Box<dyn Iterator<Item = TimeRange<'c>> + 'c> {
        let time_points = self
            .inner
            .into_iter()
            .map(|tr| {
                let a = (tr.range.start, tr.kind, tr.comments);
                let b = (tr.range.end, RuleKind::Closed, UniqueSortedVec::new());
                once(a).chain(once(b))
            })
            .flatten();

        let start = once((
            ExtendedTime::new(0, 0),
            RuleKind::Closed,
            UniqueSortedVec::new(),
        ));

        let end = once((
            ExtendedTime::new(24, 0),
            RuleKind::Closed,
            UniqueSortedVec::new(),
        ));

        let time_points = start.chain(time_points).chain(end);
        let feasibles = time_points.clone().zip(time_points.skip(1));
        let result = feasibles.filter_map(|((start, kind, comments), (end, _, _))| {
            if start < end {
                Some(TimeRange::new(start..end, kind, comments))
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

    fn insert(self, mut ins_tr: TimeRange<'c>) -> Self {
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
                    ins_tr.comments = take(&mut ins_tr.comments).union(tr.comments);
                    None
                }
            })
            .collect();

        let mut after = self
            .inner
            .into_iter()
            .filter(|tr| tr.range.end > ins_start)
            .filter_map(|mut tr| {
                tr.range.start = max(tr.range.start, ins_tr.range.end);

                if tr.range.start < tr.range.end {
                    Some(tr)
                } else {
                    ins_tr.comments = take(&mut ins_tr.comments).union(tr.comments);
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .peekable();

        // Extend the inserted interval if it has adjacent intervals with same value

        #[allow(clippy::suspicious_operation_groupings)]
        while before
            .last()
            .map(|tr| tr.range.end == ins_tr.range.start && tr.kind == ins_tr.kind)
            .unwrap_or(false)
        {
            let tr = before.pop().unwrap();
            ins_tr.range.start = tr.range.start;
            ins_tr.comments = tr.comments.union(ins_tr.comments);
        }

        #[allow(clippy::suspicious_operation_groupings)]
        while after
            .peek()
            .map(|tr| ins_tr.range.end == tr.range.start && tr.kind == ins_tr.kind)
            .unwrap_or(false)
        {
            let tr = after.next().unwrap();
            ins_tr.range.end = tr.range.end;
            ins_tr.comments = tr.comments.union(ins_tr.comments);
        }

        // Build final set of intervals

        let mut inner = before;
        inner.push(ins_tr);
        inner.extend(after);
        Schedule { inner }
    }
}

#[macro_export]
macro_rules! schedule {
    (
        $( $hh1:expr,$mm1:expr $( => $kind:expr $( , $comment:expr )* => $hh2:expr,$mm2:expr )+ );*
        $( ; )?
    ) => {{
        #[allow(unused_imports)]
        use crate::{schedule::{Schedule, TimeRange}};

        #[allow(unused_imports)]
        use opening_hours_syntax::extended_time::ExtendedTime;

        #[allow(unused_mut)]
        let mut inner = Vec::new();

        $(
            let mut prev = ExtendedTime::new($hh1, $mm1);

            $(
                let curr = ExtendedTime::new($hh2, $mm2);
                let comments = vec![$($comment),*].into();
                inner.push(TimeRange::new(prev..curr, $kind, comments));

                #[allow(unused_assignments)]
                { prev = curr }
            )+
        )*

        Schedule { inner }
    }};
}
