pub(crate) mod errors;
pub(crate) mod types;

#[cfg(test)]
mod tests;

use ::opening_hours::CoordLocation;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use self::types::InputTime;
use self::types::PyLocale;
use crate::errors::ParserError;
use crate::types::{RangeIterator, State};
use ::opening_hours::country::Country;
use ::opening_hours::{Context, OpeningHours};

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
/// Parameters
/// ----------
/// oh : str
///     Opening hours expression as defined in OSM (eg. "24/7").
///     See https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification
/// timezone : Optional[zoneinfo.ZoneInfo]
///     Timezone where the physical place attached to these opening hours lives
///     in. When specified, operations on this expression will return dates
///     attached to this timezone and input times in other timezones will be
///     converted.
/// country : Optional[str]
///     ISO code of the country this physical place lives in. This will be used
///     to load a calendar of local public holidays.
/// coords : Optional[tuple[float, float]]
///     (latitude, longitude) of this place. When this is specified together
///     with a timezone sun events will be accurate (sunrise, sunset, dusk,
///     dawn). By default, this will be used to automatically detect the
///     timezone and a country code.
/// auto_country : bool (default: `True`)
///     If set to `True`, the country code will automatically be inferred from
///     coordinates when they are specified.
/// auto_timezone : bool (default: `True`)
///     If set to `True`, the timezone will automatically be inferred from
///     coordinates when they are specified.
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
///
/// >>> dt = datetime.fromisoformat("2024-07-14 15:00")
/// >>> oh = OpeningHours("sunrise-sunset ; PH off", country="FR", coords=(48.8535, 2.34839))
/// >>> assert oh.is_closed(dt)
/// >>> assert oh.next_change(dt).replace(tzinfo=None) == datetime.fromisoformat("2024-07-15 06:03")
#[pyclass(frozen, name = "OpeningHours")]
#[derive(PartialEq)]
struct PyOpeningHours {
    inner: OpeningHours<PyLocale>,
}

#[pymethods]
impl PyOpeningHours {
    #[new]
    #[pyo3(signature = (oh, /, timezone=None, country=None, coords=None, auto_country=true, auto_timezone=true))]
    fn new(
        oh: &str,
        timezone: Option<chrono_tz::Tz>,
        country: Option<String>,
        coords: Option<(f64, f64)>,
        auto_country: Option<bool>,
        auto_timezone: Option<bool>,
    ) -> PyResult<Self> {
        let auto_country = auto_country.unwrap_or(true);
        let auto_timezone = auto_timezone.unwrap_or(true);

        let mut ctx = Context::default();
        let oh = OpeningHours::parse(oh).map_err(ParserError::from)?;

        if let Some(iso_code) = country {
            ctx = ctx.with_holidays(
                iso_code
                    .parse::<Country>()
                    .expect("unknown country") // TODO: exception
                    .holidays(),
            );
        } else if let Some((lat, lon)) = coords {
            if auto_country {
                ctx = ctx.with_holidays(Context::from_coords(lat, lon).holidays);
            }
        }

        let locale = match (timezone, coords, auto_timezone) {
            (Some(tz), None, _) | (Some(tz), _, false) => {
                PyLocale { timezone: Some(tz), coords: None }
            }
            (Some(tz), Some((lat, lon)), _) => {
                PyLocale { timezone: Some(tz), coords: Some((lat, lon)) }
            }
            (None, Some((lat, lon)), true) => {
                let tmp = CoordLocation::from_coords(lat, lon);

                PyLocale {
                    timezone: Some(*tmp.get_timezone()),
                    coords: Some((tmp.get_lat(), tmp.get_lon())),
                }
            }
            _ => PyLocale::default(),
        };

        Ok(PyOpeningHours { inner: oh.with_context(ctx.with_locale(locale)) })
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
    /// State.CLOSED
    #[pyo3(signature = (time=None, /))]
    fn state(&self, time: Option<InputTime>) -> State {
        let time = InputTime::unwrap_or_now(time);
        self.inner.state(time).into()
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
    fn is_open(&self, time: Option<InputTime>) -> bool {
        let time = InputTime::unwrap_or_now(time);
        self.inner.is_open(time)
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
    fn is_closed(&self, time: Option<InputTime>) -> bool {
        let time = InputTime::unwrap_or_now(time);
        self.inner.is_closed(time)
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
    fn is_unknown(&self, time: Option<InputTime>) -> bool {
        let time = InputTime::unwrap_or_now(time);
        self.inner.is_unknown(time)
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
    fn next_change(&self, time: Option<InputTime>) -> Option<InputTime> {
        let time = InputTime::unwrap_or_now(time);
        self.inner.next_change(time)
        // TODO: prefer input timezone
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
    /// (..., datetime.datetime(2099, 1, 1, 12, 30), State.CLOSED, [])
    /// >>> next(intervals)
    /// (datetime.datetime(2099, 1, 1, 12, 30), datetime.datetime(2099, 1, 1, 17, 0), State.OPEN, [])
    #[pyo3(signature = (start=None, end=None, /))]
    fn intervals(&self, start: Option<InputTime>, end: Option<InputTime>) -> RangeIterator {
        let start = InputTime::unwrap_or_now(start);
        RangeIterator::new(&self.inner, start, end)
        // TODO: prefer input timezone
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
    m.add_class::<State>()?;
    m.add_class::<PyOpeningHours>()?;
    Ok(())
}
