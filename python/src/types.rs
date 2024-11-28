use std::convert::Infallible;
use std::sync::OnceLock;

use chrono::{Local, NaiveDateTime};
use opening_hours::opening_hours::DATE_LIMIT;
use opening_hours::DateTimeRange;
use opening_hours_syntax::rules::RuleKind;
use pyo3::prelude::*;
use pyo3::types::PyString;

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

impl<'py> IntoPyObject<'py> for State {
    type Target = PyString;
    type Output = Borrowed<'static, 'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        static PY_CACHE_OPEN: OnceLock<Py<PyString>> = OnceLock::new();
        static PY_CACHE_CLOSED: OnceLock<Py<PyString>> = OnceLock::new();
        static PY_CACHE_UNKNOWN: OnceLock<Py<PyString>> = OnceLock::new();

        let res = match self {
            Self::Open => PY_CACHE_OPEN.get_or_init(|| PyString::new(py, "open").unbind()),
            Self::Closed => PY_CACHE_CLOSED.get_or_init(|| PyString::new(py, "closed").unbind()),
            Self::Unknown => PY_CACHE_UNKNOWN.get_or_init(|| PyString::new(py, "unknown").unbind()),
        };

        Ok(res.bind_borrowed(py))
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
