mod bindings;
mod doctests;

use crate::opening_hours;
use std::ffi::CString;
use std::fmt::Write;
use std::sync::Once;

use pyo3::prelude::*;

static INIT_PYTHON_GIL: Once = Once::new();

pub(crate) fn run_python(source: &str) {
    let common_prefix = source
        .lines()
        .filter(|l| !l.trim().is_empty())
        .reduce(|mut prefix, line| {
            while !line.starts_with(prefix) {
                prefix = &prefix[..prefix.len() - 1];
            }

            prefix
        })
        .unwrap_or("");

    let without_indent = source.lines().fold(String::new(), |mut acc, line| {
        writeln!(
            &mut acc,
            "{}",
            line.strip_prefix(common_prefix).unwrap_or("")
        )
        .unwrap();
        acc
    });

    INIT_PYTHON_GIL.call_once(|| {
        pyo3::append_to_inittab!(opening_hours);
        pyo3::prepare_freethreaded_python();
    });

    Python::with_gil(|py| {
        py.run(CString::new(without_indent).unwrap().as_c_str(), None, None)
            .unwrap();
    });
}
