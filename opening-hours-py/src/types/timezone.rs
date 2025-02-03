use pyo3::prelude::*;
use pyo3_stub_gen::{PyStubType, TypeInfo};

/// Tiny wrapper that is here to extend type with stub generation.
#[derive(FromPyObject, IntoPyObject)]
pub(crate) struct TimeZoneWrapper(chrono_tz::Tz);

impl From<TimeZoneWrapper> for chrono_tz::Tz {
    fn from(val: TimeZoneWrapper) -> Self {
        val.0
    }
}

impl PyStubType for TimeZoneWrapper {
    fn type_output() -> TypeInfo {
        TypeInfo::with_module("zoneinfo.ZoneInfo", "zoneinfo".into())
    }
}
