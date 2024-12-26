use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use opening_hours::opening_hours::DATE_LIMIT;
use opening_hours::{CoordLocation, DateTimeRange, TzLocation};
use opening_hours_syntax::rules::RuleKind;
use pyo3::prelude::*;

// ---
// --- DateTime
// ---

#[derive(FromPyObject, IntoPyObject)]
pub(crate) enum InputTime {
    Naive(NaiveDateTime),
    TzAware(DateTime<chrono_tz::Tz>),
}

impl InputTime {
    /// Just ensures that *DATE_LIMIT* is mapped to `None`.
    pub(crate) fn map_date_limit(self) -> Option<Self> {
        if self.as_naive_local() == DATE_LIMIT {
            None
        } else {
            Some(self)
        }
    }

    pub(crate) fn unwrap_or_now(val: Option<Self>) -> Self {
        val.unwrap_or_else(|| Self::Naive(Local::now().naive_local()))
    }

    pub(crate) fn as_naive_local(&self) -> NaiveDateTime {
        match self {
            InputTime::Naive(naive_date_time) => *naive_date_time,
            InputTime::TzAware(date_time) => date_time.naive_local(),
        }
    }

    pub(crate) fn as_tz_aware(&self, default_tz: &chrono_tz::Tz) -> DateTime<chrono_tz::Tz> {
        match self {
            InputTime::Naive(naive_date_time) => default_tz
                .from_local_datetime(naive_date_time)
                .earliest()
                .expect("input time is not valid for target timezone"), // TODO: exception
            InputTime::TzAware(date_time) => *date_time,
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
    iter: Box<dyn Iterator<Item = DateTimeRange<InputTime>> + Send + Sync>,
}

impl RangeIterator {
    pub fn new_naive(
        td: &opening_hours::OpeningHours,
        start: NaiveDateTime,
        end: Option<NaiveDateTime>,
    ) -> Self {
        let iter = {
            if let Some(end) = end {
                Box::new(
                    td.iter_range(start, end)
                        .map(|rg| rg.map_dates(InputTime::Naive)),
                ) as _
            } else {
                Box::new(td.iter_from(start).map(|rg| rg.map_dates(InputTime::Naive))) as _
            }
        };

        Self { iter }
    }

    pub fn new_tz_aware(
        td: &opening_hours::OpeningHours<TzLocation<chrono_tz::Tz>>,
        start: DateTime<chrono_tz::Tz>,
        end: Option<DateTime<chrono_tz::Tz>>,
    ) -> Self {
        let iter = {
            if let Some(end) = end {
                Box::new(
                    td.iter_range(start, end)
                        .map(|rg| rg.map_dates(InputTime::TzAware)),
                ) as _
            } else {
                Box::new(
                    td.iter_from(start)
                        .map(|rg| rg.map_dates(InputTime::TzAware)),
                ) as _
            }
        };

        Self { iter }
    }

    pub fn new_coords(
        td: &opening_hours::OpeningHours<CoordLocation<chrono_tz::Tz>>,
        start: DateTime<chrono_tz::Tz>,
        end: Option<DateTime<chrono_tz::Tz>>,
    ) -> Self {
        let iter = {
            if let Some(end) = end {
                Box::new(
                    td.iter_range(start, end)
                        .map(|rg| rg.map_dates(InputTime::TzAware)),
                ) as _
            } else {
                Box::new(
                    td.iter_from(start)
                        .map(|rg| rg.map_dates(InputTime::TzAware)),
                ) as _
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
