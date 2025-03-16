use std::cmp::{max, min};
use std::iter::Peekable;
use std::ops::Range;
use std::sync::Arc;

use opening_hours_syntax::{ExtendedTime, RuleKind};

/// An period of time in a schedule annotated with a state and comments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeRange {
    /// Active period for this range
    pub range: Range<ExtendedTime>,
    /// State of the schedule while this period is active
    pub kind: RuleKind,
    /// Comment raised while this period is active
    pub comment: Arc<str>,
}

impl TimeRange {
    /// Small helper to create a new range.
    pub fn new(range: Range<ExtendedTime>, kind: RuleKind, comment: Arc<str>) -> Self {
        TimeRange { range, kind, comment }
    }

    /// Extract the kind and comment from the range, which are the values that define current state
    /// of an expression.
    pub fn as_state(&self) -> (RuleKind, &str) {
        (self.kind, &self.comment)
    }
}

/// Describe a full schedule for a day, keeping track of open, closed and
/// unknown periods.
///
/// It can be turned into an iterator which will yield consecutive ranges of
/// different states, with no holes or overlapping.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Schedule {
    /// Always keep a sequence of non-overlaping, increasing time ranges.
    pub(crate) inner: Vec<TimeRange>,
}

impl Schedule {
    /// Creates a new empty schedule, which represents an always closed period.
    ///
    /// ```
    /// use opening_hours::schedule::Schedule;
    ///
    /// assert!(Schedule::new().is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new schedule from a list of ranges of same kind and comment.
    ///
    /// ```
    /// use opening_hours::schedule::Schedule;
    /// use opening_hours_syntax::{ExtendedTime, RuleKind};
    ///
    /// let sch1 = Schedule::from_ranges(
    ///     [
    ///         ExtendedTime::new(10, 0).unwrap()..ExtendedTime::new(14, 0).unwrap(),
    ///         ExtendedTime::new(12, 0).unwrap()..ExtendedTime::new(16, 0).unwrap(),
    ///     ],
    ///     RuleKind::Open,
    ///     Default::default(),
    /// );
    ///
    /// let sch2 = Schedule::from_ranges(
    ///     [ExtendedTime::new(10, 0).unwrap()..ExtendedTime::new(16, 0).unwrap()],
    ///     RuleKind::Open,
    ///     Default::default(),
    /// );
    ///
    /// assert_eq!(sch1, sch2);
    /// ```
    pub fn from_ranges(
        ranges: impl IntoIterator<Item = Range<ExtendedTime>>,
        kind: RuleKind,
        comment: Arc<str>,
    ) -> Self {
        let mut inner: Vec<_> = ranges
            .into_iter()
            .filter(|range| range.start < range.end)
            .map(|range| TimeRange { range, kind, comment: comment.clone() })
            .collect();

        // Ensure ranges are disjoint and in increasing order
        if inner.len() > 1 {
            inner.sort_unstable_by_key(|rng| rng.range.start);
            let mut i_kept = 0; // the last element to be kept

            for i_next in 1..inner.len() {
                if inner[i_kept].range.end >= inner[i_next].range.start {
                    inner[i_kept].range.end = inner[i_next].range.end;
                } else {
                    i_kept += 1;
                    inner.swap(i_kept, i_next);
                }
            }

            inner.truncate(i_kept + 1);
        }

        Self { inner }
    }

    /// Check if a schedule is empty.
    ///
    /// ```
    /// use opening_hours::schedule::Schedule;
    ///
    /// assert!(Schedule::new().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Check if a schedule is constant from 00:00 to 24:00.
    pub(crate) fn is_constant(&self) -> bool {
        self.inner.is_empty()
            || self.inner.len() == 1
                && self.inner[0].range == (ExtendedTime::MIDNIGHT_00..ExtendedTime::MIDNIGHT_24)
    }

    /// Check if a schedule is always closed with no comments.
    pub(crate) fn is_always_closed_with_no_comments(&self) -> bool {
        self.inner
            .iter()
            .all(|rg| rg.kind == RuleKind::Closed && rg.comment.is_empty())
    }

    /// Remove closed with no comment sections
    pub fn filter_closed_ranges(mut self) -> Self {
        self.inner
            .retain(|rg| rg.kind != RuleKind::Closed || !rg.comment.is_empty());

        self
    }

    /// Merge two schedules together.
    pub fn addition(self, mut other: Self) -> Self {
        // TODO (optimisation): this is implemented with quadratic time where it could probably be
        // linear.
        match other.inner.pop() {
            None => self,
            Some(tr) => self.insert(tr).addition(other),
        }
    }

    /// Insert a new time range in a schedule.
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
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .peekable();

        // Extend the inserted interval if it has adjacent intervals with same value

        while before
            .last()
            .map(|tr| tr.range.end == ins_tr.range.start && tr.as_state() == ins_tr.as_state())
            .unwrap_or(false)
        {
            let tr = before.pop().unwrap();
            ins_tr.range.start = tr.range.start;
        }

        while after
            .peek()
            .map(|tr| ins_tr.range.end == tr.range.start && tr.as_state() == ins_tr.as_state())
            .unwrap_or(false)
        {
            let tr = after.next().unwrap();
            ins_tr.range.end = tr.range.end;
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
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

/// Return value for [`Schedule::into_iter`].
#[derive(Debug)]
pub struct IntoIter {
    last_end: ExtendedTime,
    ranges: Peekable<std::vec::IntoIter<TimeRange>>,
}

impl IntoIter {
    /// The value that will fill holes
    const HOLES_STATE: (RuleKind, &str) = (RuleKind::Closed, "");

    /// Create a new iterator from a schedule.
    fn new(schedule: Schedule) -> Self {
        Self {
            last_end: ExtendedTime::MIDNIGHT_00,
            ranges: schedule.inner.into_iter().peekable(),
        }
    }

    /// Must be called before a value is yielded.
    fn pre_yield(&mut self, value: TimeRange) -> Option<TimeRange> {
        assert!(
            value.range.start < value.range.end,
            "infinite loop detected"
        );

        self.last_end = value.range.end;
        Some(value)
    }
}

impl Iterator for IntoIter {
    type Item = TimeRange;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_end >= ExtendedTime::MIDNIGHT_24 {
            // Iteration ended
            return None;
        }

        let mut yielded_range = {
            let next_start = self.ranges.peek().map(|tr| tr.range.start);

            if next_start == Some(self.last_end) {
                // Start from an interval
                self.ranges.next().unwrap()
            } else {
                // Start from a hole
                TimeRange::new(
                    self.last_end..next_start.unwrap_or(self.last_end),
                    Self::HOLES_STATE.0,
                    Default::default(),
                )
            }
        };

        while let Some(next_range) = self.ranges.peek() {
            if next_range.range.start > yielded_range.range.end {
                if yielded_range.as_state() == Self::HOLES_STATE {
                    // Just extend the closed range with this hole
                    yielded_range.range.end = next_range.range.start;
                } else {
                    // The range before the hole is not closed
                    return self.pre_yield(yielded_range);
                }
            }

            if yielded_range.as_state() != next_range.as_state() {
                // The next range has a different state
                return self.pre_yield(yielded_range);
            }

            let next_range = self.ranges.next().unwrap();
            yielded_range.range.end = next_range.range.end;
        }

        if yielded_range.as_state() == Self::HOLES_STATE {
            // Extend with the last hole
            yielded_range.range.end = ExtendedTime::MIDNIGHT_24;
        }

        self.pre_yield(yielded_range)
    }
}

impl std::iter::FusedIterator for IntoIter {}

/// Macro that allows to quickly create a complex schedule.
///
/// ## Syntax
///
/// You can define multiple sequences of time as follows :
///
/// ```plain
/// {time_0} => {state_1} => {time_2} => {state_2} => ... => {state_n} => {time_n};
/// ```
///
/// Where the time values are written `{hour},{minutes}` and states are a
/// [`RuleKind`] value, optionally followed by a list of comment literals.
///
/// ```
/// use opening_hours_syntax::{ExtendedTime, RuleKind};
///
/// opening_hours::schedule! {
///      9,00 => RuleKind::Open => 12,00;
///     14,00 => RuleKind::Open => 18,00
///           => RuleKind::Unknown, "Closes when stock is depleted" => 20,00;
///     22,00 => RuleKind::Closed, "Maintenance team only" => 26,00;
/// };
/// ```
#[macro_export]
macro_rules! schedule {
    (
        $( $hh1:expr,$mm1:expr $( => $kind:expr $( , $comment:expr )? => $hh2:expr,$mm2:expr )+ );*
        $( ; )?
    ) => {{
        #[allow(unused_imports)]
        use $crate::schedule::{Schedule, TimeRange};

        #[allow(unused_imports)]
        use opening_hours_syntax::extended_time::ExtendedTime;

        #[allow(unused_mut)]
        let mut schedule = Schedule::new();

        $(
            let mut prev = ExtendedTime::new($hh1, $mm1)
                .expect("Invalid interval start");

            $(
                let curr = ExtendedTime::new($hh2, $mm2)
                    .expect("Invalid interval end");

                let comment = [$(std::sync::Arc::from($comment))?].into_iter().next().unwrap_or_default();
                let next_schedule = Schedule::from_ranges([prev..curr], $kind, comment);
                schedule = schedule.addition(next_schedule);

                #[allow(unused_assignments)]
                { prev = curr }
            )+
        )*

        schedule
    }};
}
