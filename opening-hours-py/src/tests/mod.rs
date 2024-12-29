mod bindings;
mod doctests;

use crate::opening_hours;
use std::ffi::CString;
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, Once};

use pyo3::prelude::*;

pub(crate) fn test_fn_name(source: &str) -> String {
    let mut hash = std::hash::DefaultHasher::new();
    source.hash(&mut hash);
    format!("test_python_{:x}", hash.finish())
}

pub(crate) fn wrap_with_function(source: &str) -> String {
    let fn_name = test_fn_name(source);
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

    let mut output = source
        .lines()
        .fold(format!("def {fn_name}():"), |mut acc, line| {
            writeln!(
                &mut acc,
                "    {}",
                line.strip_prefix(common_prefix).unwrap_or(""),
            )
            .unwrap();
            acc
        });

    writeln!(&mut output, "{fn_name}()").unwrap();
    output
}

pub(crate) fn run_python(source: &str) {
    static INIT_PYTHON_GIL: Once = Once::new();

    // Capture stdout into a buffer
    #[pyclass]
    #[derive(Clone)]
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

    // Remove indentation and wrap test into a function
    let source_wrapped = wrap_with_function(source);

    // Start GIL once per session
    INIT_PYTHON_GIL.call_once(|| {
        pyo3::append_to_inittab!(opening_hours);
        pyo3::prepare_freethreaded_python();
    });

    // Run test
    Python::with_gil(|py| {
        let sys = py.import("sys").expect("could not import sys");
        let buffer: Arc<Mutex<String>> = Arc::default();

        sys.setattr(
            "stdout",
            CaptureStdout(buffer.clone()).into_pyobject(py).unwrap(),
        )
        .expect("could not intercept stdout");

        sys.setattr(
            "stderr",
            CaptureStdout(buffer.clone()).into_pyobject(py).unwrap(),
        )
        .expect("could not intercept stderr");

        if let Err(err) = py.run(CString::new(source_wrapped).unwrap().as_c_str(), None, None) {
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
