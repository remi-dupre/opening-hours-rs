pub(crate) mod errors;
pub(crate) mod types;

#[cfg(test)]
mod tests;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use self::types::InputTime;
use crate::errors::ParserError;
use crate::types::{RangeIterator, State};
use ::opening_hours::country::Country;
use ::opening_hours::{Context, CoordLocation, Localize, NoLocation, OpeningHours, TzLocation};

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

#[derive(PartialEq)]
enum PyOpeningHoursInner {
    Naive(OpeningHours<NoLocation>),
    WithTz {
        oh: OpeningHours<TzLocation<chrono_tz::Tz>>,
        tz: chrono_tz::Tz,
    },
    WithCoords {
        oh: OpeningHours<CoordLocation<chrono_tz::Tz>>,
        tz: chrono_tz::Tz,
    },
}

impl From<PyOpeningHoursInner> for PyOpeningHours {
    fn from(val: PyOpeningHoursInner) -> Self {
        PyOpeningHours { inner: val }
    }
}

/// Parse input opening hours description.
///
/// TODO: explaine, country, coords, timezone, ...
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
#[pyclass(frozen, name = "OpeningHours")]
#[derive(PartialEq)]
struct PyOpeningHours {
    inner: PyOpeningHoursInner,
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

        Ok(match (timezone, coords, auto_timezone) {
            (Some(tz), None, _) | (Some(tz), _, false) => {
                let ctx = ctx.with_locale(TzLocation::new(tz));
                PyOpeningHoursInner::WithTz { oh: oh.with_context(ctx), tz }.into()
            }
            (Some(tz), Some((lat, lon)), _) => {
                let ctx = ctx.with_locale(CoordLocation { tz, lat, lon });
                PyOpeningHoursInner::WithCoords { oh: oh.with_context(ctx), tz }.into()
            }
            (None, Some((lat, lon)), true) => {
                let locale = NoLocation::default().with_tz_from_coords(lat, lon);
                let tz = locale.tz;
                let ctx = ctx.with_locale(locale);
                PyOpeningHoursInner::WithCoords { oh: oh.with_context(ctx), tz }.into()
            }
            _ => PyOpeningHoursInner::Naive(oh.with_context(ctx)).into(),
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
    // #[pyo3(text_signature = "(self, time=None, /)")]
    #[pyo3(signature = (time=None, /))]
    fn state(&self, time: Option<InputTime>) -> State {
        let time = InputTime::unwrap_or_now(time);

        match &self.inner {
            PyOpeningHoursInner::Naive(oh) => oh.state(time.as_naive_local()).into(),
            PyOpeningHoursInner::WithTz { oh, tz } => oh.state(time.as_tz_aware(tz)).into(),
            PyOpeningHoursInner::WithCoords { oh, tz } => oh.state(time.as_tz_aware(tz)).into(),
        }
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

        match &self.inner {
            PyOpeningHoursInner::Naive(oh) => oh.is_open(time.as_naive_local()),
            PyOpeningHoursInner::WithTz { oh, tz } => oh.is_open(time.as_tz_aware(tz)),
            PyOpeningHoursInner::WithCoords { oh, tz } => oh.is_open(time.as_tz_aware(tz)),
        }
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

        match &self.inner {
            PyOpeningHoursInner::Naive(oh) => oh.is_closed(time.as_naive_local()),
            PyOpeningHoursInner::WithTz { oh, tz } => oh.is_closed(time.as_tz_aware(tz)),
            PyOpeningHoursInner::WithCoords { oh, tz } => oh.is_closed(time.as_tz_aware(tz)),
        }
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

        match &self.inner {
            PyOpeningHoursInner::Naive(oh) => oh.is_unknown(time.as_naive_local()),
            PyOpeningHoursInner::WithTz { oh, tz } => oh.is_closed(time.as_tz_aware(tz)),
            PyOpeningHoursInner::WithCoords { oh, tz } => oh.is_closed(time.as_tz_aware(tz)),
        }
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

        match &self.inner {
            PyOpeningHoursInner::Naive(oh) => InputTime::Naive(
                oh.next_change(time.as_naive_local())
                    .expect("unexpected date beyond year 10 000"),
            ),
            PyOpeningHoursInner::WithTz { oh, tz } => InputTime::TzAware(
                oh.next_change(time.as_tz_aware(tz))
                    .expect("unexpected date beyond year 10 000"),
            ),
            PyOpeningHoursInner::WithCoords { oh, tz } => InputTime::TzAware(
                oh.next_change(time.as_tz_aware(tz))
                    .expect("unexpected date beyond year 10 000"),
            ),
        }
        .map_date_limit()
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
    fn intervals(&self, start: Option<InputTime>, end: Option<InputTime>) -> RangeIterator {
        let start = InputTime::unwrap_or_now(start);

        match &self.inner {
            PyOpeningHoursInner::Naive(oh) => RangeIterator::new_naive(
                oh,
                start.as_naive_local(),
                end.map(|dt| dt.as_naive_local()),
            ),
            PyOpeningHoursInner::WithTz { oh, tz } => RangeIterator::new_tz_aware(
                oh,
                start.as_tz_aware(tz),
                end.map(|dt| dt.as_tz_aware(tz)),
            ),
            PyOpeningHoursInner::WithCoords { oh, tz } => RangeIterator::new_coords(
                oh,
                start.as_tz_aware(tz),
                end.map(|dt| dt.as_tz_aware(tz)),
            ),
        }
    }

    #[pyo3()]
    fn __str__(&self) -> String {
        match &self.inner {
            PyOpeningHoursInner::Naive(oh) => oh.to_string(),
            PyOpeningHoursInner::WithTz { oh, tz: _ } => oh.to_string(),
            PyOpeningHoursInner::WithCoords { oh, tz: _ } => oh.to_string(),
        }
    }

    #[pyo3()]
    fn __repr__(&self) -> String {
        match &self.inner {
            PyOpeningHoursInner::Naive(oh) => {
                format!("OpeningHours({:?})", oh.to_string())
            }
            PyOpeningHoursInner::WithTz { oh, tz } => {
                format!("OpeningHours({:?}, tz={tz})", oh.to_string())
            }
            PyOpeningHoursInner::WithCoords { oh, tz } => {
                format!("OpeningHours({:?}, tz={tz})", oh.to_string())
            }
        }
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
