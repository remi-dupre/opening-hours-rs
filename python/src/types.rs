use std::ops::Add;

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeDelta, TimeZone};
use opening_hours::opening_hours::DATE_LIMIT;
use opening_hours::{
    CoordLocation, DateTimeRange, Localize, LocalizeWithTz, NoLocation, TzLocation,
};
use opening_hours_syntax::rules::time::TimeEvent;
use opening_hours_syntax::rules::RuleKind;
use pyo3::prelude::*;

// ---
// --- DateTime
// ---

#[derive(Clone, FromPyObject, IntoPyObject)]
pub(crate) enum InputTime {
    Naive(NaiveDateTime),
    TzAware(DateTime<chrono_tz::Tz>),
}

impl InputTime {
    fn as_naive_local(&self) -> NaiveDateTime {
        match self {
            InputTime::Naive(naive_date_time) => *naive_date_time,
            InputTime::TzAware(date_time) => date_time.naive_local(),
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
}

impl Add<TimeDelta> for InputTime {
    type Output = Self;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        match self {
            InputTime::Naive(dt) => InputTime::Naive(dt + rhs),
            InputTime::TzAware(dt) => InputTime::TzAware(dt + rhs),
        }
    }
}

// ---
// --- Localization
// ---

#[derive(Copy, Clone, Default, PartialEq)]
pub(crate) struct PyLocale {
    pub(crate) timezone: Option<chrono_tz::Tz>,
    pub(crate) coords: Option<(f64, f64)>,
}

impl Localize for PyLocale {
    type DateTime = InputTime;

    type WithTz<T>
        = Self
    where
        T: TimeZone + Send + Sync,
        T::Offset: Send + Sync;

    fn naive(&self, dt: Self::DateTime) -> NaiveDateTime {
        match dt {
            InputTime::Naive(dt) => dt,
            InputTime::TzAware(dt) => {
                if let Some(local_tz) = self.timezone {
                    dt.with_timezone(&local_tz).naive_local()
                } else {
                    dt.naive_local()
                }
            }
        }
    }

    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime {
        if let Some(local_tz) = self.timezone {
            InputTime::TzAware(TzLocation { tz: local_tz }.datetime(naive))
        } else {
            InputTime::Naive(naive)
        }
    }

    fn event_time(&self, date: NaiveDate, event: TimeEvent) -> chrono::NaiveTime {
        if let (Some(tz), Some((lat, lon))) = (self.timezone, self.coords) {
            CoordLocation { tz, lat, lon }.event_time(date, event)
        } else {
            NoLocation::default().event_time(date, event)
        }
    }

    fn with_tz<T>(self, _tz: T) -> Self::WithTz<T>
    where
        T: TimeZone + Send + Sync,
        T::Offset: Send + Sync,
    {
        self
    }
}

impl LocalizeWithTz for PyLocale {
    type WithCoord = Self;

    fn with_coord(self, lat: f64, lon: f64) -> Self::WithCoord {
        Self { coords: Some((lat, lon)), ..self }
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
    iter: Box<dyn Iterator<Item = DateTimeRange<InputTime>> + Send + Sync>,
}

impl RangeIterator {
    pub(crate) fn new(
        td: &opening_hours::OpeningHours<PyLocale>,
        start: InputTime,
        end: Option<InputTime>,
    ) -> Self {
        let iter = {
            if let Some(end) = end {
                Box::new(td.iter_range(start, end)) as _
            } else {
                Box::new(td.iter_from(start)) as _
            }
        };

        Self { iter }
    }
}

#[pymethods]
impl RangeIterator {
    fn __iter__(slf: PyRef<Self>) -> Py<Self> {
        slf.into()
    }

    fn __next__(
        mut slf: PyRefMut<Self>,
    ) -> Option<(InputTime, Option<InputTime>, State, Vec<String>)> {
        let dt_range = slf.iter.next()?;

        Some((
            dt_range.range.start,
            dt_range.range.end.map_date_limit(),
            dt_range.kind.into(),
            dt_range.comments.iter().map(|c| c.to_string()).collect(),
        ))
    }
}
