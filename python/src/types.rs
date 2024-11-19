use chrono::{Local, NaiveDateTime};
use opening_hours::opening_hours::DATE_LIMIT;
use pyo3::prelude::*;

use opening_hours::DateTimeRange;
use opening_hours_syntax::rules::RuleKind;

pub(crate) fn get_time(datetime: Option<NaiveDateTime>) -> NaiveDateTime {
    datetime.unwrap_or_else(|| Local::now().naive_local())
}

pub(crate) fn res_time(datetime: NaiveDateTime) -> Option<NaiveDateTime> {
    if datetime == DATE_LIMIT {
        None
    } else {
        Some(datetime)
    }
}

// ---
// --- State
// ---

pub enum State {
    Open,
    Closed,
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

impl IntoPy<Py<PyAny>> for State {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        match self {
            Self::Open => "open".into_py(py),
            Self::Closed => "closed".into_py(py),
            Self::Unknown => "unknown".into_py(py),
        }
    }
}

// ---
// --- RangeIterator
// ---

/// Iterator that owns a pointer to a [`OpeningHours`] together with a
/// self reference to it.
#[pyclass()]
pub struct RangeIterator {
    iter: Box<dyn Iterator<Item = DateTimeRange> + Send + Sync>,
}

impl RangeIterator {
    pub fn new(
        td: &opening_hours::OpeningHours,
        start: NaiveDateTime,
        end: Option<NaiveDateTime>,
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
    ) -> Option<(NaiveDateTime, Option<NaiveDateTime>, State, Vec<String>)> {
        let dt_range = slf.iter.next()?;

        Some((
            dt_range.range.start,
            res_time(dt_range.range.end),
            dt_range.kind.into(),
            dt_range.comments.iter().map(|c| c.to_string()).collect(),
        ))
    }
}
