use std::fmt;

use pyo3::exceptions::PySyntaxError;
use pyo3::prelude::*;

// --
// -- Parsing errors
// --

#[derive(Debug)]
pub struct ParserError(opening_hours::ParserError);

impl std::error::Error for ParserError {}

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
