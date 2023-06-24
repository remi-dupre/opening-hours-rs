use std::fmt;

use pyo3::exceptions::{PySyntaxError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDateTime;

/// Error while parsing OH expression
#[derive(Debug)]
pub struct OhError(opening_hours::error::Error);

impl std::error::Error for OhError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl fmt::Display for OhError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not parse expression:\n{}", self.0)
    }
}

impl From<opening_hours::error::Error> for OhError {
    fn from(parser_error: opening_hours::error::Error) -> Self {
        Self(parser_error)
    }
}

impl From<OhError> for PyErr {
    fn from(error: OhError) -> Self {
        match error.0 {
            opening_hours::error::Error::Parser(inner) => PySyntaxError::new_err(inner.to_string()),
            opening_hours::error::Error::DateLimitExceeded(_)
            | opening_hours::error::Error::RegionNotFound(_)
            | opening_hours::error::Error::TzNotFound(_) => {
                PySyntaxError::new_err(error.0.to_string())
            }
        }
    }
}

/// Error while converting a Python date to a Rust date
#[derive(Debug)]
pub struct DateImportError<'d>(pub(crate) &'d PyDateTime);

impl std::error::Error for DateImportError<'_> {}

impl fmt::Display for DateImportError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "datetime not supported by Rust: {}", self.0)
    }
}

impl From<DateImportError<'_>> for PyErr {
    fn from(date_import_error: DateImportError) -> Self {
        PyValueError::new_err(date_import_error.to_string())
    }
}
