//! # ![][demo-favicon] Python bindings for [OSM Opening Hours](https://github.com/remi-dupre/opening-hours-rs)
//!
//! [![PyPI](https://img.shields.io/pypi/v/opening-hours-py)][pypi]
//! [![Doc](https://img.shields.io/badge/doc-pdoc-blue)][docs]
//! [![PyPI - Downloads](https://img.shields.io/pypi/dm/opening-hours-py)][pypi]
//! [![Coverage](https://img.shields.io/codecov/c/github/remi-dupre/opening-hours-rs)][codecov]
//! [![][demo-button]][demo-website]
//!
//! ## Usage
//!
//! The pre-compiled package is published for Python 3.9 and above and new releases
//! will adapt to [officially supported Python versions][python-versions].
//!
//! If you want to install this library with older version of Python, **you will
//! need the Rust toolchain** (`rustc` and `cargo`).
//!
//! Install `opening-hours-py` from PyPI, for example using pip:
//!
//! ```bash
//! pip install --user opening-hours-py
//! ```
//!
//! Then, the main object that you will interact with will be `OpeningHours`:
//!
//! ```python
//! from opening_hours import OpeningHours
//!
//! oh = OpeningHours("Mo-Fr 10:00-18:00; Sa-Su 10:00-12:00")
//! print("Current status is", oh.state())
//! print("This will change at", oh.next_change())
//!
//! # You can also attach a timezone to your expression. If you use timezone-aware
//! # dates, they will be converted to local time before any computation is done.
//! from zoneinfo import ZoneInfo
//! oh = OpeningHours("Mo-Fr 10:00-18:00; Sa-Su 10:00-12:00", timezone=ZoneInfo("Europe/Paris"))
//!
//! # The timezone can also be infered with coordinates
//! oh = OpeningHours("Mo-Fr 10:00-18:00; Sa-Su 10:00-12:00", coords=(48.8535, 2.34839))
//!
//! # You can normalize the expression
//! assert str(OpeningHours("24/7 ; Su closed").normalize()) == "Mo-Sa"
//! ```
//!
//! The API is very similar to Rust API but you can find a Python specific
//! documentation [here](https://remi-dupre.github.io/opening-hours-rs/opening_hours.html).
//!
//! ## Features
//!
//! - 📝 Parsing for [OSM opening hours][grammar]
//! - 🧮 Evaluation of state and next change
//! - ⏳ Lazy infinite iterator
//! - 🌅 Accurate sun events
//! - 📅 Embedded public holidays database for many countries (from [nager])
//! - 🌍 Timezone support
//! - 🔥 Fast and memory-safe implementation using Rust
//! - 📏 [Normalization][docs-normalize] to unambiguous expressions
//!
//! ## Limitations
//!
//! Expressions will always be considered closed **before 1900 and after 9999**.
//! This comes from the specification not supporting date outside of this grammar
//! and makes the implementation slightly more convenient.
//!
//! Feel free to open an issue if you have a use case for extreme dates!
//!
//! ## Development
//!
//! To build the library by yourself you will require a recent version of Rust,
//! [`rustup`](https://www.rust-lang.org/tools/install) is usually the recommended
//! tool to manage the installation.
//!
//! Then you can use poetry to install Python dependencies and run `maturin` (the
//! building tool used to create the bindings) from a virtualenv.
//!
//! ```bash
//! $ git clone https://github.com/remi-dupre/opening-hours-rs.git
//! $ cd opening-hours-rs
//!
//! # Install Python dependancies
//! $ poetry install --with dev
//!
//! # Enter the virtualenv
//! $ poetry shell
//!
//! # Build developpement bindings, add `--release` for an optimized version
//! $ maturin develop
//!
//! # Now the library is available as long as you don't leave the virtualenv
//! $ python
//! >>> from opening_hours import OpeningHours
//! >>> oh = OpeningHours("24/7")
//! >>> oh.state()
//! "open"
//! ```
//!
//! [codecov]: https://app.codecov.io/gh/remi-dupre/opening-hours-rs "Code coverage"
//! [demo-button]: https://raw.githubusercontent.com/remi-dupre/opening-hours-demo/refs/heads/main/static/demo-button.svg
//! [demo-favicon]: https://raw.githubusercontent.com/remi-dupre/opening-hours-demo/refs/heads/main/static/favicon.ico "icon"
//! [demo-website]: https://remi-dupre.github.io/opening-hours-demo/ "Demonstration website"
//! [docs]: https://remi-dupre.github.io/opening-hours-rs/opening_hours.html "Documentation"
//! [docs-normalize]: https://remi-dupre.github.io/opening-hours-rs/opening_hours.html#OpeningHours.normalize "Normalization documentation"
//! [grammar]: https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification "OSM specification for opening hours"
//! [nager]: https://date.nager.at/api/v3 "Worldwide holidays (REST API)"
//! [pypi]: https://pypi.org/project/opening-hours-py/ "PyPI page"
//! [python-versions]: https://devguide.python.org/versions/#supported- "Python release cycle"

// NOTE: pyo3-stub-gen ignores inner macro in #[doc = include_str!("...")] so we need to copy
// instead of including markdown files.
// See https://github.com/Jij-Inc/pyo3-stub-gen/issues/485

pub(crate) mod types;

#[cfg(test)]
mod tests;

use std::str::FromStr;

use chrono::TimeDelta;
use opening_hours_syntax::Parser;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};

use opening_hours_rs::localization::{Coordinates, Country, TzLocation};
use opening_hours_rs::{Context, OpeningHours};

use crate::types::datetime::DateTimeMaybeAware;
use crate::types::iterator::RangeIterator;
use crate::types::location::PyLocation;
use crate::types::state::State;
use crate::types::timezone::TimeZoneWrapper;

pyo3::create_exception!(
    opening_hours,
    ParserError,
    PyException,
    concat!(
        "The opening hours expression has an invalid syntax.\n",
        "\n",
        "See https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification\n",
        "for a specification.",
    )
);

pyo3::create_exception!(
    opening_hours,
    UnknownCountryError,
    PyException,
    concat!(
        "The provided country code is not known.\n",
        "\n",
        "See https://en.wikipedia.org/wiki/List_of_ISO_3166_country_codes.",
    )
);

pyo3::create_exception!(
    opening_hours,
    InvalidCoordinatesError,
    PyException,
    concat!("Input coordinates are not valid.")
);

/// Validate that input string is a correct opening hours description.
///
/// ## Examples
///
/// >>> opening_hours.validate("24/7")
/// True
/// >>> opening_hours.validate("24/24")
/// False
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(text_signature = "(oh, /)")]
fn validate(oh: &str) -> bool {
    OpeningHours::from_str(oh).is_ok()
}

/// Parse input opening hours description.
///
/// ## Parameters
///
/// - oh: Opening hours expression as defined in OSM (eg. "24/7"). See
///   https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification
/// - timezone: Timezone where the physical place attached to these opening hours lives in. When
///   specified, operations on this expression will return dates attached to this timezone and
///   input times in other timezones will be converted.
/// - country: ISO code of the country this physical place lives in. This will be used to load a
///   calendar of local public holidays.
/// - coords: (latitude, longitude) of this place. When this is specified together with a timezone
///   sun events will be accurate (sunrise, sunset, dusk, dawn). By default, this will be used to
///   automatically detect the timezone and a country code.
/// - auto_country: If set to `True`, the country code will automatically be inferred from
///   coordinates when they are specified.
/// - auto_timezone: If set to `True`, the timezone will automatically be inferred from coordinates
///   when they are specified.
/// - max_interval_days: If specified, any change that is longer than the number of specified days
///   will be considered infinite. This may be useful if you need to evaluate a large amount of
///   complicated expressions and performance is critical. Even setting a value of a full year (366)
///   is worth it.
///
/// ## Raises
///
/// SyntaxError
///     Given string is not in valid opening hours format.
///
/// ## Examples
///
/// >>> oh = OpeningHours("24/7")
/// >>> oh.is_open()
/// True
///
/// >>> dt = datetime.fromisoformat("2024-07-14 15:00")
/// >>> oh = OpeningHours("sunrise-sunset ; PH off", country="FR", coords=(48.8535, 2.34839))
/// >>> assert oh.is_closed(dt)
/// >>> assert oh.next_change(dt).replace(tzinfo=None) == datetime.fromisoformat("2024-07-15 06:03")
#[gen_stub_pyclass]
#[pyclass(frozen, name = "OpeningHours")]
#[derive(PartialEq)]
struct PyOpeningHours {
    inner: OpeningHours<PyLocation>,
    warnings: Vec<String>,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyOpeningHours {
    #[new]
    #[pyo3(signature = (oh, timezone=None, country=None, coords=None, auto_country=Some(true), auto_timezone=Some(true), max_interval_days=None))]
    fn new(
        oh: &str,
        timezone: Option<TimeZoneWrapper>,
        country: Option<String>,
        coords: Option<(f64, f64)>,
        auto_country: Option<bool>,
        auto_timezone: Option<bool>,
        max_interval_days: Option<u32>,
    ) -> PyResult<Self> {
        let auto_country = auto_country.unwrap_or(true);
        let auto_timezone = auto_timezone.unwrap_or(true);

        let mut warnings = Vec::new();
        let mut ctx = Context::default();

        if let Some(days) = max_interval_days {
            ctx = ctx.approx_bound_interval_size(TimeDelta::days(days.into()))
        }

        let mut parser =
            Parser::default().with_warning_handler(|warning| warnings.push(warning.to_string()));

        let coords = coords
            .map(|(lat, lon)| {
                Coordinates::new(lat, lon).ok_or_else(|| {
                    InvalidCoordinatesError::new_err(format!("Invalid coordinates ({lat}, {lon})"))
                })
            })
            .transpose()?;

        let oh = OpeningHours::parse_with(&mut parser, oh)
            .map_err(|err| ParserError::new_err(format!("Failed to parse expression: {err}")))?;

        if let Some(iso_code) = country {
            ctx = ctx.with_holidays(
                iso_code
                    .parse::<Country>()
                    .map_err(|err| UnknownCountryError::new_err(err.to_string()))?
                    .holidays(),
            );
        } else if auto_country && let Some(coords) = coords {
            ctx = ctx.with_holidays(Context::from_coords(coords).holidays);
        }

        let locale = match (timezone, coords, auto_timezone) {
            (Some(tz), None, _) | (Some(tz), _, false) => {
                PyLocation::Aware(TzLocation::new(tz.into()))
            }
            (Some(tz), Some(coords), _) => {
                PyLocation::Aware(TzLocation::new(tz.into()).with_coords(coords))
            }
            (None, Some(coords), true) => PyLocation::Aware(TzLocation::from_coords(coords)),
            _ => PyLocation::Naive,
        };

        Ok(PyOpeningHours {
            inner: oh.with_context(ctx.with_locale(locale)),
            warnings,
        })
    }

    /// The list of warnings that were emited while parsing the expression.
    #[getter]
    fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }

    /// Convert the expression into a normalized form. It will not affect the meaning of the
    /// expression and might impact the performance of evaluations.
    ///
    /// ## Examples
    ///
    /// >>> OpeningHours("24/7 ; Su closed").normalize()
    /// OpeningHours("Mo-Sa")
    ///
    /// # Motivation
    ///
    /// Normalization attempts to transform an expression into a minimal sequence of
    /// _non-overlapping_, normal rules. The goal is _not_ to make the expression
    /// shorter but instead to make as readable as possible. For example, the
    /// additional operator `,` is less known and can be mistaken with any other kind
    /// of sequence (eg. in a day selector `Mo,Fr`).
    ///
    /// Normalization is [_idempotent_][wiki-idempotence], which means that normalizing
    /// an already normalized expression won't change the result.
    ///
    /// ## Examples
    ///
    /// | input                                          | normalized                                                                  |
    /// | ---------------------------------------------- | --------------------------------------------------------------------------- |
    /// | `Mo-Su 00:00-24:00`                            | `24/7`                                                                      |
    /// | `24/7 ; Su closed`                             | `Mo-Sa`                                                                     |
    /// | `Mo-Su 10:00-12:00, Mo-Fr 14:00-18:00`         | `Mo-Fr 10:00-12:00,14:00-18:00; Sa-Su 10:00-12:00`                          |
    /// | `10:00-18:00; Jul-Aug 10:00-22:00`             | `Jan-Jun,Sep-Dec 10:00-18:00; Jul-Aug 10:00-22:00`                          |
    /// | `Mo-Fr,Su 10:00-18:00; Jul-Aug Su 10:00-22:00` | `Mo-Fr 10:00-18:00; Jan-Jun,Sep-Dec Su 10:00-18:00; Jul-Aug Su 10:00-22:00` |
    ///
    /// ## Unsupported syntax
    ///
    /// Not all syntax can be normalized, but this library will still do some best
    /// effort by normalizing the longest prefix possible and keeping all rules after
    /// the first unsupported one unchanged.
    ///
    /// Here is an exhausting list of the kind of syntax you can't expect to see
    /// normalized by current implementation:
    ///
    /// | kind                                                    | behavior                   | example (1)                  |
    /// | ------------------------------------------------------- | -------------------------- | ---------------------------- |
    /// | [fallback rule][spec-fallback]                          | stop normalization (2)     | `Mo-Fr \|\| unknown`         |
    /// | any range with steps                                    | stop normalization (2)     | `2000-3000/5`                |
    /// | [monthday range][spec-monthday-range] with fixed dates  | stop normalization (2)     | `Mar31-Jun01`                |
    /// | [monthday range][spec-monthday-range] with year         | stop normalization (2)     | `2025Jun-Aug`                |
    /// | [weekday range][spec-weekday-range] with index in month | stop normalization (2)     | `Mo[2]`, `Mo[2] +1 days`     |
    /// | [weekday range][spec-weekday-range] with a holiday      | stop normalization (2)     | `easter`                     |
    /// | time that overlaps with next day                        | stop normalization (2)     | `22:00-06:00`, `22:00-28:00` |
    /// | time with a solar event                                 | no time simplification (3) | `sunrise-18:00`              |
    /// | time with an open end                                   | no time simplification (3) | `12:00-16:00+`               |
    /// | time with repetition                                    | no time simplification (3) | `12:00-16:00/02:00`          |
    ///
    /// Notes :
    ///
    /// 1. All the examples above contain a single rule, so they would be left
    ///    unchanged by the normalization.
    /// 2. This rule and any following rule won't be treated.
    /// 3. This won't halt normalization but the algorithm won't try to merge this time
    ///    range with others.
    ///
    /// If a feature is not implemented I may have considered it to be too niche for
    /// the effort. Feel free to [open an issue][gh-issues] on Github or open a merge
    /// request if you disagree!
    ///
    /// # How it works
    ///
    /// ## Build a canonical time table
    ///
    /// First, create a "canonical" time table over 4 dimensions (year, month, weeknum,
    /// daynum), each cell keeps track of time ranges recorded for a single combination
    /// of intervals over those 4 dimensions. Cells are always non-overlapping and can
    /// be split while processing the expression if necessary.
    ///
    /// For example, the resulting structure looks like this (simplified to 2
    /// dimensions for obvious reasons):
    ///
    /// ```text
    ///     Mo    Sa  Su
    /// Jan ╆━━━━━┪───┢━━━┪     Expression:
    ///     ┃ (1) ┃   ┃(1)┃     Mo-Fr,Su 10:00-18:00; Jul-Aug Su 10:00-22:00
    /// Jul ┨╌╌╌╌╌┃───┣━━━┫
    ///     ┃ (1) ┃   ┃(2)┃     Time rules:
    /// Sep ┨╌╌╌╌╌┃───┣━━━┫     (1) 10:00-18:00
    ///     ┃ (1) ┃   ┃(1)┃     (2) 10:00-22:00
    ///     ┗━━━━━┛───┗━━━┛
    /// ```
    ///
    /// ## Extract covering rectangles out of the table
    ///
    /// Second, the algorithm will extract maximal rectangle in the table with all
    /// inner cells equal to the same value.
    ///
    /// ```text
    /// Step 1: extracted a rectangle
    /// - weekday: Mo-Fr
    /// - month: Jan-Dec
    /// - time: 10:00-18:00
    ///
    ///     Mo    Sa  Su
    /// Jan ╆━━━━━┪───┢━━━┓     Expression:
    ///     ┃▚▚▚▚▚┃   ┃(1)┃     Mo-Fr,Su 10:00-18:00; Jul-Aug Su 10:00-22:00
    /// Jul ┨▚▚▚▚▚┃───┣━━━┫
    ///     ┃▚▚▚▚▚┃   ┃(2)┃     Time rules:
    /// Sep ┨▚▚▚▚▚┃───┣━━━┫     (1) 10:00-18:00
    ///     ┃▚▚▚▚▚┃   ┃(1)┃     (2) 10:00-22:00
    ///     ┗━━━━━┛───┗━━━┛
    ///
    /// Step 2: extracted a rectangle
    /// - weekday: Su
    /// - month: Jan-Jun,Sep-Dec
    /// - time: 10:00-18:00
    ///
    ///     Mo        Su
    /// Jan ┼─────────┢━━━┓     Expression:
    ///     │         ┃▚▚▚┃     Mo-Fr,Su 10:00-18:00; Jul-Aug Su 10:00-22:00
    /// Jul ┤         ┣━━━┫
    ///     │         ┃(2)┃     Time rules:
    /// Sep ┤         ┣━━━┫     (1) 10:00-18:00
    ///     │         ┃▚▚▚┃     (2) 10:00-22:00
    ///     └─────────┗━━━┛
    ///
    /// Step 3: extracted a rectangle
    /// - weekday: Su
    /// - month: Jul-Aug
    /// - time: 10:00-22:00
    ///
    ///     Mo        Su
    ///     ├─────────┼───┐     Expression:
    ///     │         │   │     Mo-Fr,Su 10:00-18:00; Jul-Aug Su 10:00-22:00
    /// Jul ┤         ┏━━━┓
    ///     │         ┃▚▚▚┃     Time rules:
    /// Sep ┤         ┗━━━┛     (1) 10:00-18:00
    ///     │         │   │     (2) 10:00-22:00
    ///     └─────────┴───┘
    /// ```
    ///
    /// The result is then the concatenation : `Mo-Fr 10:00-18:00; Jan-Jun,Sep-Dec Su
    /// 10:00-18:00; Jul-Aug Su 10:00-22:00`.
    ///
    /// [gh-issues]: https://github.com/remi-dupre/opening-hours-rs/issues
    /// [spec-fallback]: https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification#fallback_rule_separator
    /// [spec-monthday-range]: https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification#monthday_range
    /// [spec-weekday-range]: https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification#weekday_range
    /// [wiki-idempotence]: https://en.wikipedia.org/wiki/Idempotence
    fn normalize(&self) -> Self {
        PyOpeningHours {
            inner: self.inner.normalize(),
            warnings: Vec::default(),
        }
    }

    /// Get current state of the time domain together with current comment. The state can be either
    /// "open", "closed" or "unknown".
    ///
    /// ## Parameters
    ///
    /// - time: Base time for the evaluation, current time will be used if it is not specified.
    ///
    /// ## Examples
    ///
    /// >>> OpeningHours("24/7 off").state()
    /// (State.CLOSED, '')
    #[pyo3(signature = (time=None))]
    fn state(&self, time: Option<DateTimeMaybeAware>) -> (State, String) {
        let time = DateTimeMaybeAware::unwrap_or_now(time);
        let (kind, comment) = self.inner.state(time);
        (kind.into(), comment.to_string())
    }

    /// Check if current state is open.
    ///
    /// ## Parameters
    ///
    /// - time: Base time for the evaluation, current time will be used if it is not specified.
    ///
    /// ## Examples
    ///
    /// >>> OpeningHours("24/7").is_open()
    /// True
    #[pyo3(signature = (time=None))]
    fn is_open(&self, time: Option<DateTimeMaybeAware>) -> bool {
        let time = DateTimeMaybeAware::unwrap_or_now(time);
        self.inner.is_open(time)
    }

    /// Check if current state is closed.
    ///
    /// ## Parameters
    ///
    /// - time: Base time for the evaluation, current time will be used if it is not specified.
    ///
    /// ## Examples
    ///
    /// >>> OpeningHours("24/7 off").is_closed()
    /// True
    #[pyo3(signature = (time=None))]
    fn is_closed(&self, time: Option<DateTimeMaybeAware>) -> bool {
        let time = DateTimeMaybeAware::unwrap_or_now(time);
        self.inner.is_closed(time)
    }

    /// Check if current state is unknown.
    ///
    /// ## Parameters
    ///
    /// - time: Base time for the evaluation, current time will be used if it is not specified.
    ///
    /// ## Examples
    ///
    /// >>> OpeningHours("24/7 unknown").is_unknown()
    /// True
    #[pyo3(signature = (time=None))]
    fn is_unknown(&self, time: Option<DateTimeMaybeAware>) -> bool {
        let time = DateTimeMaybeAware::unwrap_or_now(time);
        self.inner.is_unknown(time)
    }

    /// Get the date for next change of state.
    /// If the date exceed the limit date, returns None.
    ///
    /// ## Parameters
    ///
    /// - time: Base time for the evaluation, current time will be used if it is not specified.
    ///
    /// ## Examples
    ///
    /// >>> OpeningHours("24/7").next_change() # None
    /// >>> OpeningHours("2099Mo-Su 12:30-17:00").next_change()
    /// datetime.datetime(2099, 1, 1, 12, 30)
    #[pyo3(signature = (time=None))]
    fn next_change(&self, time: Option<DateTimeMaybeAware>) -> Option<DateTimeMaybeAware> {
        let time = DateTimeMaybeAware::unwrap_or_now(time);

        self.inner
            .next_change(time)
            .map(|dt| dt.or_with_timezone_of(time))
    }

    /// Give an iterator that yields successive time intervals of consistent
    /// state.
    ///
    /// ## Parameters
    ///
    /// - start: Initial time for the iterator, current time will be used if it is not specified.
    /// - end: Maximal time for the iterator, the iterator will continue until year 9999 if it no
    ///   max is specified.
    ///
    /// ## Examples
    ///
    /// >>> intervals = OpeningHours("2099Mo-Su 12:30-17:00").intervals()
    /// >>> next(intervals)
    /// (..., datetime.datetime(2099, 1, 1, 12, 30), State.CLOSED, '')
    /// >>> next(intervals)
    /// (datetime.datetime(2099, 1, 1, 12, 30), datetime.datetime(2099, 1, 1, 17, 0), State.OPEN, '')
    #[pyo3(signature = (start=None, end=None))]
    fn intervals(
        &self,
        start: Option<DateTimeMaybeAware>,
        end: Option<DateTimeMaybeAware>,
    ) -> RangeIterator {
        let start = DateTimeMaybeAware::unwrap_or_now(start);
        RangeIterator::new(&self.inner, start, end)
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
fn opening_hours(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    m.add(
        "InvalidCoordinatesError",
        py.get_type::<InvalidCoordinatesError>(),
    )?;

    m.add("ParserError", py.get_type::<ParserError>())?;
    m.add("UnknownCountryError", py.get_type::<UnknownCountryError>())?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_class::<State>()?;
    m.add_class::<PyOpeningHours>()?;
    Ok(())
}

pub fn stub_info() -> pyo3_stub_gen::Result<pyo3_stub_gen::StubInfo> {
    let manifest_dir: &::std::path::Path = env!("CARGO_MANIFEST_DIR").as_ref();

    pyo3_stub_gen::StubInfo::from_pyproject_toml(
        manifest_dir
            .parent()
            .expect("could not locate crate root")
            .join("pyproject.toml"),
    )
}
