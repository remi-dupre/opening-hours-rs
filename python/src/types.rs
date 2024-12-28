use std::ops::Add;

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeDelta};
use opening_hours::opening_hours::DATE_LIMIT;
use opening_hours::{DateTimeRange, Localize, NoLocation, TzLocation};
use opening_hours_syntax::rules::time::TimeEvent;
use opening_hours_syntax::rules::RuleKind;
use pyo3::prelude::*;

// ---
// --- DateTime
// ---

#[derive(Clone, Copy, FromPyObject, IntoPyObject)]
pub(crate) enum DateTimeMaybeAware {
    Naive(NaiveDateTime),
    Aware(DateTime<chrono_tz::Tz>),
}

impl DateTimeMaybeAware {
    /// Drop eventual timezone information.
    pub(crate) fn as_naive_local(&self) -> NaiveDateTime {
        match self {
            DateTimeMaybeAware::Naive(naive_date_time) => *naive_date_time,
            DateTimeMaybeAware::Aware(date_time) => date_time.naive_local(),
        }
    }

    /// Just ensures that *DATE_LIMIT* is mapped to `None`.
    pub(crate) fn map_date_limit(self) -> Option<Self> {
        if self.as_naive_local() == DATE_LIMIT {
            None
        } else {
            Some(self)
        }
    }

    /// Fetch local time if value is `None`.
    pub(crate) fn unwrap_or_now(val: Option<Self>) -> Self {
        val.unwrap_or_else(|| Self::Naive(Local::now().naive_local()))
    }

    pub(crate) fn timezone(&self) -> Option<chrono_tz::Tz> {
        match self {
            Self::Naive(_) => None,
            Self::Aware(dt) => Some(dt.timezone()),
        }
    }

    pub(crate) fn or_with_timezone(self, tz: chrono_tz::Tz) -> Self {
        match self {
            Self::Naive(dt) => Self::Aware(TzLocation::new(tz).datetime(dt)),
            Self::Aware(_) => self,
        }
    }

    pub(crate) fn or_with_timezone_of(self, other: Self) -> Self {
        match other {
            Self::Naive(_) => self,
            Self::Aware(dt) => self.or_with_timezone(dt.timezone()),
        }
    }
}

impl Add<TimeDelta> for DateTimeMaybeAware {
    type Output = Self;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        match self {
            DateTimeMaybeAware::Naive(dt) => DateTimeMaybeAware::Naive(dt + rhs),
            DateTimeMaybeAware::Aware(dt) => DateTimeMaybeAware::Aware(dt + rhs),
        }
    }
}

// ---
// --- Localization
// ---

#[derive(Clone, PartialEq)]
pub(crate) enum PyLocation {
    Naive,
    Aware(TzLocation<chrono_tz::Tz>),
}

impl Localize for PyLocation {
    type DateTime = DateTimeMaybeAware;

    fn naive(&self, dt: Self::DateTime) -> NaiveDateTime {
        match self {
            PyLocation::Naive => NoLocation.naive(dt.as_naive_local()),
            PyLocation::Aware(loc) => match dt {
                DateTimeMaybeAware::Naive(dt) => dt,
                DateTimeMaybeAware::Aware(dt) => loc.naive(dt),
            },
        }
    }

    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime {
        match self {
            Self::Naive => DateTimeMaybeAware::Naive(NoLocation.datetime(naive)),
            Self::Aware(loc) => DateTimeMaybeAware::Aware(loc.datetime(naive)),
        }
    }

    fn event_time(&self, date: NaiveDate, event: TimeEvent) -> chrono::NaiveTime {
        match self {
            PyLocation::Naive => NoLocation.event_time(date, event),
            PyLocation::Aware(loc) => loc.event_time(date, event),
        }
    }
}

// ---
// --- State
// ---

/// Specify the state of an opening hours interval.
#[pyclass(ord, eq, frozen, hash, str, rename_all = "UPPERCASE")]
#[derive(Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum State {
    /// Currently open
    Open,
    /// Currently closed
    Closed,
    /// May be open depending on context
    Unknown,
}

impl From<RuleKind> for State {
    fn from(kind: RuleKind) -> Self {
        match kind {
            RuleKind::Open => Self::Open,
            RuleKind::Closed => Self::Closed,
            RuleKind::Unknown => Self::Unknown,
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Open => write!(f, "open"),
            State::Closed => write!(f, "closed"),
            State::Unknown => write!(f, "unknown"),
        }
    }
}

// ---
// --- RangeIterator
// ---

/// Iterator over a range period of an [`OpeningHours`].
#[pyclass()]
pub struct RangeIterator {
    prefer_timezone: Option<chrono_tz::Tz>,
    iter: Box<dyn Iterator<Item = DateTimeRange<DateTimeMaybeAware>> + Send + Sync>,
}

impl RangeIterator {
    pub(crate) fn new(
        td: &opening_hours::OpeningHours<PyLocation>,
        start: DateTimeMaybeAware,
        end: Option<DateTimeMaybeAware>,
    ) -> Self {
        let iter = {
            if let Some(end) = end {
                Box::new(td.iter_range(start, end)) as _
            } else {
                Box::new(td.iter_from(start)) as _
            }
        };

        Self {
            prefer_timezone: start
                .timezone()
                .or_else(|| end.and_then(|dt| dt.timezone())),
            iter,
        }
    }

    fn map_prefered_timezone(&self, dt: DateTimeMaybeAware) -> DateTimeMaybeAware {
        if let Some(tz) = self.prefer_timezone {
            dt.or_with_timezone(tz)
        } else {
            dt
        }
    }
}

#[pymethods]
impl RangeIterator {
    fn __iter__(slf: PyRef<Self>) -> Py<Self> {
        slf.into()
    }

    fn __next__(
        mut slf: PyRefMut<Self>,
    ) -> Option<(
        DateTimeMaybeAware,
        Option<DateTimeMaybeAware>,
        State,
        Vec<String>,
    )> {
        let dt_range = slf.iter.next()?;

        Some((
            slf.map_prefered_timezone(dt_range.range.start),
            slf.map_prefered_timezone(dt_range.range.end)
                .map_date_limit(),
            dt_range.kind.into(),
            dt_range.comments.iter().map(|c| c.to_string()).collect(),
        ))
    }
}
