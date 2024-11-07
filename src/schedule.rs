use std::cmp::{max, min};
use std::iter::Peekable;
use std::mem::take;
use std::ops::Range;
use std::sync::Arc;

use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::RuleKind;
use opening_hours_syntax::sorted_vec::UniqueSortedVec;

/// TODO: doc
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeRange {
    pub range: Range<ExtendedTime>,
    pub kind: RuleKind,
    pub comments: UniqueSortedVec<Arc<str>>,
}

impl TimeRange {
    pub(crate) fn new(
        range: Range<ExtendedTime>,
        kind: RuleKind,
        comments: UniqueSortedVec<Arc<str>>,
    ) -> Self {
        TimeRange { range, kind, comments }
    }
}

/// Describe a full schedule for a day, keeping track of open, closed and
/// unknown periods.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Schedule {
    /// Always keep a sequence of non-overlaping, increasing time ranges.
    pub(crate) inner: Vec<TimeRange>,
}

impl Schedule {
    /// Creates a new empty schedule, which represents an always closed period.
    ///
    /// # Example
    ///
    /// ```
    /// use opening_hours::schedule::Schedule;
    ///
    /// assert!(Schedule::empty().is_empty());
    /// ```
    pub fn empty() -> Self {
        Self::default()
    }

    /// TODO: doc
    pub fn from_ranges(
        ranges: impl IntoIterator<Item = Range<ExtendedTime>>,
        kind: RuleKind,
        comments: &UniqueSortedVec<Arc<str>>,
    ) -> Self {
        Schedule {
            inner: ranges
                .into_iter()
                .inspect(|range| assert!(range.start < range.end))
                .map(|range| TimeRange::new(range, kind, comments.clone()))
                .collect(),
        }
    }

    /// TODO: doc
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// TODO: doc
    pub fn into_iter_filled(self) -> FilledScheduleIterator {
        FilledScheduleIterator::new(self)
    }

    /// TODO: doc
    pub fn addition(self, mut other: Self) -> Self {
        // TODO: this is implemented with quadratic time where it could probably
        //       be linear.
        match other.inner.pop() {
            None => self,
            Some(tr) => self.insert(tr).addition(other),
        }
    }

    /// TODO: doc
    fn insert(self, mut ins_tr: TimeRange) -> Self {
        // Build sets of intervals before and after the inserted interval

        let ins_start = ins_tr.range.start;
        let ins_end = ins_tr.range.end;

        let mut before: Vec<_> = self
            .inner
            .iter()
            .filter(|tr| tr.range.start < ins_end)
            .cloned()
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

impl IntoIterator for Schedule {
    type Item = TimeRange;
    type IntoIter = <Vec<TimeRange> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

/// Return value for [`Schedule::into_iter_filled`].
#[derive(Debug)]
pub struct FilledScheduleIterator {
    last_end: ExtendedTime,
    ranges: Peekable<<Schedule as IntoIterator>::IntoIter>,
}

impl FilledScheduleIterator {
    /// The value that will fill holes
    const HOLES_STATE: RuleKind = RuleKind::Closed;

    /// First minute of the schedule
    const START_TIME: ExtendedTime = ExtendedTime::new(0, 0);

    /// Last minute of the schedule
    const END_TIME: ExtendedTime = ExtendedTime::new(24, 0);

    /// Create a new iterator from a schedule
    fn new(schedule: Schedule) -> Self {
        Self {
            last_end: Self::START_TIME,
            ranges: schedule.into_iter().peekable(),
        }
    }

    /// Must be called before a value is yielded
    fn pre_yield(&mut self, value: TimeRange) -> Option<TimeRange> {
        assert!(
            value.range.start < value.range.end,
            "infinite loop detected"
        );

        self.last_end = value.range.end;
        Some(value)
    }
}

impl Iterator for FilledScheduleIterator {
    type Item = TimeRange;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_end >= Self::END_TIME {
            return None;
        }

        let mut yielded_range = {
            let next_start = self.ranges.peek().map(|tr| tr.range.start);

            if next_start == Some(self.last_end) {
                self.ranges.next().unwrap()
            } else {
                TimeRange::new(
                    self.last_end..next_start.unwrap_or(self.last_end),
                    Self::HOLES_STATE,
                    UniqueSortedVec::new(),
                )
            }
        };

        while let Some(next_range) = self.ranges.peek() {
            if next_range.range.start > yielded_range.range.end {
                if yielded_range.kind == Self::HOLES_STATE {
                    // Just extend the closed range with this hole
                    yielded_range.range.end = next_range.range.start;
                } else {
                    // The range before the hole is not closed
                    return self.pre_yield(yielded_range);
                }
            }

            if yielded_range.kind != next_range.kind {
                return self.pre_yield(yielded_range);
            }

            let next_range = self.ranges.next().unwrap();
            yielded_range.range.end = next_range.range.end;
            yielded_range.comments = yielded_range.comments.union(next_range.comments);
        }

        if yielded_range.kind == Self::HOLES_STATE {
            yielded_range.range.end = Self::END_TIME;
        }

        self.pre_yield(yielded_range)
    }
}

/// Macro to ease the creation of schedules during for unit tests.
#[cfg(test)]
#[macro_export]
macro_rules! schedule {
    (
        $( $hh1:expr,$mm1:expr $( => $kind:expr $( , $comment:expr )* => $hh2:expr,$mm2:expr )+ );*
        $( ; )?
    ) => {{
        #[allow(unused_imports)]
        use $crate::{schedule::{Schedule, TimeRange}};

        #[allow(unused_imports)]
        use opening_hours_syntax::extended_time::ExtendedTime;

        #[allow(unused_mut)]
        let mut inner = Vec::new();

        $(
            let mut prev = ExtendedTime::new($hh1, $mm1);

            $(
                let curr = ExtendedTime::new($hh2, $mm2);
                let comments = vec![$(std::sync::Arc::from($comment)),*].into();
                inner.push(TimeRange::new(prev..curr, $kind, comments));

                #[allow(unused_assignments)]
                { prev = curr }
            )+
        )*

        Schedule { inner }
    }};
}
