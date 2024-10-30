use std::collections::HashMap;

use chrono::Duration;
use once_cell::sync::Lazy;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDelta, PyDeltaAccess, PyTzInfo};

#[derive(Debug)]
pub(crate) enum PyTz {
    Native(chrono_tz::Tz),
    Delta(chrono::FixedOffset),
}

pub(crate) fn tz_from_python(py: Python<'_>, tzinfo: &PyAny) -> PyResult<PyTz> {
    if let Ok(tzname) = tzinfo.extract::<&str>() {
        TZ_BY_NAME
            .get(tzname)
            .copied()
            .map(PyTz::Native)
            .ok_or_else(|| PyValueError::new_err(format!("Unknown TimeZone '{tzname}'")))
    } else if let Ok(tzinfo) = tzinfo.extract::<&PyTzInfo>() {
        // // Attempts to fetch `zone` attribute from `pytz` package
        if let Ok(py_tz_name) = tzinfo.getattr("zone") {
            let tz_name: &str = py_tz_name.extract()?;

            if let Some(tz) = TZ_BY_NAME.get(tz_name) {
                return Ok(PyTz::Native(*tz));
            }
        }

        py.import("datetime")?;
        let dt = py.eval("datetime.datetime.now()", None, None)?;

        // Fallback to offset
        let tz_delta: &PyDelta = tzinfo.call_method("utcoffset", (dt,), None)?.extract()?;

        let duration = Duration::microseconds(
            i64::from(tz_delta.get_microseconds())
                + 1_000_000
                    * (i64::from(tz_delta.get_seconds())
                        + 24 * 3600 * i64::from(tz_delta.get_days())),
        );

        Ok(PyTz::Delta(
            chrono::FixedOffset::east_opt(
                duration
                    .num_seconds()
                    .try_into()
                    .expect("invalid offset seconds"),
            )
            .expect("invalid offset value"),
        ))
    } else {
        Err(PyValueError::new_err("`tzinfo` must be a str or a tzinfo"))
    }
}
