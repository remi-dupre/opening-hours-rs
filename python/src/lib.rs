mod errors;
mod types;

use std::pin::Pin;
use std::sync::Arc;

use chrono::offset::Local;
use chrono::NaiveDateTime;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use types::RangeIterator;

use crate::errors::ParserError;
use crate::types::{NaiveDateTimeWrapper, State};

fn get_time(datetime: Option<NaiveDateTime>) -> NaiveDateTime {
    datetime.unwrap_or_else(|| Local::now().naive_local())
}

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
    opening_hours::OpeningHours::parse(oh).is_ok()
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
#[pyclass]
#[pyo3(text_signature = "(oh, /)")]
struct OpeningHours {
    inner: Pin<Arc<opening_hours::OpeningHours>>,
}

#[pymethods]
impl OpeningHours {
    #[new]
    fn new(oh: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Arc::pin(opening_hours::OpeningHours::parse(oh).map_err(ParserError::from)?),
        })
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
    #[pyo3(text_signature = "(self, time=None, /)")]
    fn state(&self, time: Option<NaiveDateTimeWrapper>) -> State {
        self.inner
            .state(get_time(time.map(Into::into)))
            .expect("unexpected date beyond year 10 000")
            .into()
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
    #[pyo3(text_signature = "(self, time=None, /)")]
    fn is_open(&self, time: Option<NaiveDateTimeWrapper>) -> bool {
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
    #[pyo3(text_signature = "(self, time=None, /)")]
    fn is_closed(&self, time: Option<NaiveDateTimeWrapper>) -> bool {
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
    #[pyo3(text_signature = "(self, time=None, /)")]
    fn is_unknown(&self, time: Option<NaiveDateTimeWrapper>) -> bool {
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
    #[pyo3(text_signature = "(self, time=None, /)")]
    fn next_change(&self, time: Option<NaiveDateTimeWrapper>) -> NaiveDateTimeWrapper {
        self.inner
            .next_change(get_time(time.map(Into::into)))
            .expect("unexpected date beyond year 10 000")
            .into()
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
    #[pyo3(text_signature = "(self, start=None, end=None, /)")]
    fn intervals(
        &self,
        start: Option<NaiveDateTimeWrapper>,
        end: Option<NaiveDateTimeWrapper>,
    ) -> RangeIterator {
        RangeIterator::new(
            self.inner.clone(),
            get_time(start.map(Into::into)),
            end.map(Into::into),
        )
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
fn opening_hours(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(validate, m)?).unwrap();
    m.add_class::<OpeningHours>()?;
    Ok(())
}
