use std::fmt;

use pyo3::exceptions::{PySyntaxError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDateTime;

/// Error while parsing OH expression
#[derive(Debug)]
pub struct ParserError(opening_hours::ParserError);

impl std::error::Error for ParserError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not parse expression:\n{}", self.0)
    }
}

impl From<opening_hours::ParserError> for ParserError {
    fn from(parser_error: opening_hours::ParserError) -> Self {
        Self(parser_error)
    }
}

impl From<ParserError> for PyErr {
    fn from(parser_error: ParserError) -> Self {
        PySyntaxError::new_err(parser_error.to_string())
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
