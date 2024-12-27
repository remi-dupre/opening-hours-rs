mod bindings;
mod doctests;

use crate::opening_hours;
use std::ffi::CString;
use std::fmt::Write;
use std::sync::{Arc, LazyLock, Mutex, Once};

use pyo3::prelude::*;

static INIT_PYTHON_GIL: Once = Once::new();

pub(crate) fn run_python(source: &str) {
    static GIL_LOCK: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

    #[pyclass]
    struct CaptureStdout(Arc<Mutex<String>>);

    #[pymethods]
    impl CaptureStdout {
        #[classattr]
        fn encoding() -> &'static str {
            "utf-8"
        }

        fn write(&self, data: &str) {
            self.0.lock().unwrap().push_str(data);
        }
    }

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

    let _guard = GIL_LOCK.lock().expect("could not get exclusive GIL lock");

    Python::with_gil(|py| {
        let sys = py.import("sys").expect("could not import sys");
        let buffer: Arc<Mutex<String>> = Arc::default();

        sys.setattr(
            "stdout",
            CaptureStdout(buffer.clone()).into_pyobject(py).unwrap(),
        )
        .expect("could not intercept stdout");

        if let Err(err) = py.run(CString::new(without_indent).unwrap().as_c_str(), None, None) {
            let traceback = err
                .traceback(py)
                .and_then(|tb| tb.format().ok())
                .map(|s| "\n".to_string() + &s)
                .unwrap_or_default();

            println!("=== Captured stdout ===");
            println!("{}", buffer.lock().unwrap());
            println!("=== Captured stdout (end) ===");
            panic!("Python Error ({err:?}): {traceback}")
        }
    });
}
