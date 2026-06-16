use std::ffi::CString;

use chrono::{DateTime, NaiveDateTime};

pub(crate) fn string_into_c_lossy(string: String) -> CString {
    let mut bytes = string.into_bytes();
    bytes.retain(|c| *c != 0);

    // Safety: we just ensured that there is no null char in bytes
    unsafe { CString::from_vec_unchecked(bytes) }
}

pub(crate) fn read_timestamp(ts: i64) -> Option<NaiveDateTime> {
    // We want to reserve values bellow 0 for empty responses or errors.
    if ts <= 0 {
        return None;
    }

    let dt = DateTime::from_timestamp_secs(ts)?;
    Some(dt.naive_utc())
}
