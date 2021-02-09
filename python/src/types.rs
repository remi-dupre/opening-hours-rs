use std::convert::TryInto;
use std::pin::Pin;
use std::sync::Arc;

use chrono::prelude::*;
use chrono::NaiveDateTime;
use pyo3::prelude::*;
use pyo3::types::{PyDateAccess, PyDateTime, PyTimeAccess};
use pyo3::PyIterProtocol;

use opening_hours::time_domain;
use opening_hours::time_domain::{DateTimeRange, RuleKind};

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

impl<'p> IntoPy<Py<PyAny>> for State {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        match self {
            Self::Open => "open".into_py(py),
            Self::Closed => "closed".into_py(py),
            Self::Unknown => "unknown".into_py(py),
        }
    }
}

// ---
// --- NaiveDateTime wrapper
// ---

pub struct NaiveDateTimeWrapper(NaiveDateTime);

impl Into<NaiveDateTime> for NaiveDateTimeWrapper {
    fn into(self) -> NaiveDateTime {
        self.0
    }
}

impl From<NaiveDateTime> for NaiveDateTimeWrapper {
    fn from(dt: NaiveDateTime) -> Self {
        Self(dt)
    }
}

impl<'source> FromPyObject<'source> for NaiveDateTimeWrapper {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let py_datetime: &PyDateTime = ob.downcast()?;
        Ok({
            NaiveDateTime::new(
                NaiveDate::from_ymd(
                    py_datetime.get_year(),
                    py_datetime.get_month().into(),
                    py_datetime.get_day().into(),
                ),
                NaiveTime::from_hms(
                    py_datetime.get_hour().into(),
                    py_datetime.get_minute().into(),
                    py_datetime.get_second().into(),
                ),
            )
            .into()
        })
    }
}

impl<'p> IntoPy<PyResult<Py<PyDateTime>>> for NaiveDateTimeWrapper {
    fn into_py(self, py: Python<'_>) -> PyResult<Py<PyDateTime>> {
        PyDateTime::new(
            py,
            self.0.date().year(),
            self.0.date().month().try_into()?,
            self.0.date().day().try_into()?,
            self.0.time().hour().try_into()?,
            self.0.time().minute().try_into()?,
            0,
            0,
            None,
        )
        .map(|x| x.into())
    }
}

impl<'p> IntoPy<Py<PyAny>> for NaiveDateTimeWrapper {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        let result: PyResult<_> = self.into_py(py);
        result
            .expect("failed at converting Rust date to Python")
            .into_py(py)
    }
}

// ---
// --- RangeIterator
// ---

/// Iterator that owns a pointer to a [`time_domain::TimeDomain`] together with a
/// self reference to it.
#[pyclass(unsendable)]
pub struct RangeIterator {
    _td: Pin<Arc<time_domain::TimeDomain>>,
    iter: Box<dyn Iterator<Item = DateTimeRange<'static>>>,
}

impl RangeIterator {
    pub fn new(
        td: Pin<Arc<time_domain::TimeDomain>>,
        start: NaiveDateTime,
        end: Option<NaiveDateTime>,
    ) -> Self {
        let iter = {
            if let Some(end) = end {
                Box::new(
                    td.iter_range(start, end)
                        .expect("unexpected date beyond year 10 000"),
                ) as _
            } else {
                Box::new(
                    td.iter_from(start)
                        .expect("unexpected date beyond year 10 000"),
                ) as _
            }
        };

        // Extend the lifetime of the reference to td inside of iter.
        //   1. `td` won't be dropped before `iter` as they are both owned by the struct.
        //   2. `td` won't move as it is marked Pin.
        //   3. we must ensure in [`RangeIterator`]'s implementation that iter is not moved out of
        //      the struct.
        let iter: Box<dyn Iterator<Item = DateTimeRange<'_>>> = iter;
        let iter: Box<dyn Iterator<Item = DateTimeRange<'static>>> =
            unsafe { std::mem::transmute(iter) };

        Self { _td: td, iter }
    }
}

#[pyproto]
impl PyIterProtocol for RangeIterator {
    fn __iter__(slf: PyRef<Self>) -> Py<RangeIterator> {
        slf.into()
    }

    fn __next__(
        mut slf: PyRefMut<Self>,
    ) -> Option<(
        NaiveDateTimeWrapper,
        NaiveDateTimeWrapper,
        State,
        Vec<&'p str>,
    )> {
        let dt_range = slf.iter.next()?;
        Some((
            dt_range.range.start.into(),
            dt_range.range.end.into(),
            dt_range.kind.into(),
            dt_range.comments,
        ))
    }
}
