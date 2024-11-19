mod errors;
mod types;

use chrono::NaiveDateTime;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use self::types::{get_time, res_time};
use crate::errors::ParserError;
use crate::types::{RangeIterator, State};
use ::opening_hours::OpeningHours;

/// Validate that input string is a correct opening hours description.
///
/// Examples
/// --------
/// >>> opening_hours.validate("24/7")
/// True
/// >>> opening_hours.validate("24/24")
/// False
#[pyfunction]
#[pyo3(text_signature = "(oh, /)")]
fn validate(oh: &str) -> bool {
    OpeningHours::parse(oh).is_ok()
}

/// Parse input opening hours description.
///
/// Raises
/// ------
/// SyntaxError
///     Given string is not in valid opening hours format.
///
/// Examples
/// --------
/// >>> oh = OpeningHours("24/7")
/// >>> oh.is_open()
/// True
#[pyclass(eq, hash, frozen, name = "OpeningHours")]
#[derive(Hash, PartialEq, Eq)]
struct PyOpeningHours {
    inner: OpeningHours,
}

#[pymethods]
impl PyOpeningHours {
    #[new]
    #[pyo3(signature = (oh, /))]
    fn new(oh: &str) -> PyResult<Self> {
        Ok(Self { inner: oh.parse().map_err(ParserError::from)? })
    }

    /// Get current state of the time domain, the state can be either "open",
    /// "closed" or "unknown".
    ///
    /// Parameters
    /// ----------
    /// time : Optional[datetime]
    ///     Base time for the evaluation, current time will be used if it is
    ///     not specified.
    ///
    /// Examples
    /// --------
    /// >>> OpeningHours("24/7 off").state()
    /// 'closed'
    // #[pyo3(text_signature = "(self, time=None, /)")]
    #[pyo3(signature = (time=None, /))]
    fn state(&self, time: Option<NaiveDateTime>) -> State {
        self.inner.state(get_time(time.map(Into::into))).into()
    }

    /// Check if current state is open.
    ///
    /// Parameters
    /// ----------
    /// time : Optional[datetime]
    ///     Base time for the evaluation, current time will be used if it is
    ///     not specified.
    ///
    /// Examples
    /// --------
    /// >>> OpeningHours("24/7").is_open()
    /// True
    #[pyo3(signature = (time=None, /))]
    fn is_open(&self, time: Option<NaiveDateTime>) -> bool {
        self.inner.is_open(get_time(time.map(Into::into)))
    }

    /// Check if current state is closed.
    ///
    /// Parameters
    /// ----------
    /// time : Optional[datetime]
    ///     Base time for the evaluation, current time will be used if it is
    ///     not specified.
    ///
    /// Examples
    /// --------
    /// >>> OpeningHours("24/7 off").is_closed()
    /// True
    #[pyo3(signature = (time=None, /))]
    fn is_closed(&self, time: Option<NaiveDateTime>) -> bool {
        self.inner.is_closed(get_time(time.map(Into::into)))
    }

    /// Check if current state is unknown.
    ///
    /// Parameters
    /// ----------
    /// time : Optional[datetime]
    ///     Base time for the evaluation, current time will be used if it is
    ///     not specified.
    ///
    /// Examples
    /// --------
    /// >>> OpeningHours("24/7 unknown").is_unknown()
    /// True
    #[pyo3(signature = (time=None, /))]
    fn is_unknown(&self, time: Option<NaiveDateTime>) -> bool {
        self.inner.is_unknown(get_time(time.map(Into::into)))
    }

    /// Get the date for next change of state.
    /// If the date exceed the limit date, returns None.
    ///
    /// Parameters
    /// ----------
    /// time : Optional[datetime]
    ///     Base time for the evaluation, current time will be used if it is
    ///     not specified.
    ///
    /// Examples
    /// --------
    /// >>> OpeningHours("24/7").next_change() # None
    /// >>> OpeningHours("2099Mo-Su 12:30-17:00").next_change()
    /// datetime.datetime(2099, 1, 1, 12, 30)
    #[pyo3(signature = (time=None, /))]
    fn next_change(&self, time: Option<NaiveDateTime>) -> Option<NaiveDateTime> {
        res_time(
            self.inner
                .next_change(get_time(time))
                .expect("unexpected date beyond year 10 000"),
        )
    }

    /// Give an iterator that yields successive time intervals of consistent
    /// state.
    ///
    /// Parameters
    /// ----------
    /// start: Optional[datetime]
    ///     Initial time for the iterator, current time will be used if it is
    ///     not specified.
    /// end : Optional[datetime]
    ///     Maximal time for the iterator, the iterator will continue until
    ///     year 9999 if it no max is specified.
    ///
    /// Examples
    /// --------
    /// >>> intervals = OpeningHours("2099Mo-Su 12:30-17:00").intervals()
    /// >>> next(intervals)
    /// (..., datetime.datetime(2099, 1, 1, 12, 30), 'closed', [])
    /// >>> next(intervals)
    /// (datetime.datetime(2099, 1, 1, 12, 30), datetime.datetime(2099, 1, 1, 17, 0), 'open', [])
    #[pyo3(signature = (start=None, end=None, /))]
    fn intervals(&self, start: Option<NaiveDateTime>, end: Option<NaiveDateTime>) -> RangeIterator {
        RangeIterator::new(
            &self.inner,
            get_time(start.map(Into::into)),
            end.map(Into::into),
        )
    }

    #[pyo3()]
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    #[pyo3()]
    fn __repr__(&self) -> String {
        format!("OpeningHours({:?})", self.inner.to_string())
    }
}

/// A library for parsing and working with OSM's opening hours field. You can
/// find its specification [here](https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification)
/// and the reference JS library [here](https://github.com/opening-hours/opening_hours.js).
///
/// Note that the specification is quite messy and that the JS library takes
/// liberty to extend it quite a lot. This means that most of the real world data
/// don't actually comply to the very restrictive grammar detailed in the official
/// specification. This library tries to fit with the real world data while
/// remaining as close as possible to the core specification.
///
/// The main structure you will have to interact with is OpeningHours, which
/// represents a parsed definition of opening hours.
#[pymodule]
fn opening_hours(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_class::<PyOpeningHours>()?;
    Ok(())
}
