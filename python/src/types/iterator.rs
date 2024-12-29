use pyo3::prelude::*;

use opening_hours::DateTimeRange;

use super::datetime::DateTimeMaybeAware;
use super::location::PyLocation;
use super::state::State;

/// Iterator over a range period of an [`OpeningHours`].
#[pyclass]
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
