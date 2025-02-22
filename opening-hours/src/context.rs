use std::fmt::Debug;
use std::sync::Arc;

use compact_calendar::CompactCalendar;

use crate::localization::{Localize, NoLocation};

// --
// -- Holidays
// --

/// Pairs a set of public holidays with a set of school holidays.
#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct ContextHolidays {
    pub(crate) public: Arc<CompactCalendar>,
    pub(crate) school: Arc<CompactCalendar>,
}

impl ContextHolidays {
    /// Create a new holidays context from sets of public and school holidays.
    pub fn new(public: Arc<CompactCalendar>, school: Arc<CompactCalendar>) -> Self {
        Self { public, school }
    }

    /// Get the set of public holidays attached to this context.
    pub fn get_public(&self) -> &CompactCalendar {
        &self.public
    }

    /// Get the set of school holidays attached to this context.
    pub fn get_school(&self) -> &CompactCalendar {
        &self.school
    }
}

/// All the context attached to a parsed OpeningHours expression and that can
/// alter its evaluation semantics.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Context<L = NoLocation> {
    /// A calendar use for evaluation of public and private holidays.
    pub holidays: ContextHolidays,
    /// Specify locality of the place attached to the expression: from
    /// timezone to coordinates.
    pub locale: L,
    /// As an approximation, consider that any interval bigger that this size
    /// is infinite. This can be enabled if you need better performance and
    /// you don't care if a shop is open in more than a year.
    pub approx_bound_interval_size: Option<chrono::TimeDelta>,
}

impl<L> Context<L> {
    /// Attach a new holidays component to this context.
    pub fn with_holidays(self, holidays: ContextHolidays) -> Self {
        Self { holidays, ..self }
    }

    /// Attach a new locale component to this context.
    pub fn with_locale<L2: Localize>(self, locale: L2) -> Context<L2> {
        Context {
            holidays: self.holidays,
            locale,
            approx_bound_interval_size: None,
        }
    }

    /// Enables appromiation of long intervals.
    pub fn approx_bound_interval_size(self, max_size: chrono::TimeDelta) -> Self {
        Self { approx_bound_interval_size: Some(max_size), ..self }
    }
}

#[cfg(feature = "auto-timezone")]
impl Context<crate::localization::TzLocation<chrono_tz::Tz>> {
    /// Create a context with given coordinates and try to infer a timezone and
    /// a local holiday calendar.
    ///
    /// ```
    /// use opening_hours::Context;
    /// use opening_hours::localization::{Coordinates, Country, TzLocation};
    ///
    /// let coords = Coordinates::new(48.8535, 2.34839).unwrap();
    ///
    /// assert_eq!(
    ///     Context::from_coords(coords),
    ///     Context::default()
    ///         .with_holidays(Country::FR.holidays())
    ///         .with_locale(TzLocation::from_coords(coords)),
    /// );
    /// ```
    #[cfg(feature = "auto-country")]
    pub fn from_coords(coords: crate::localization::Coordinates) -> Self {
        use crate::localization::Country;

        let holidays = Country::try_from_coords(coords)
            .map(Country::holidays)
            .unwrap_or_default();

        let locale = crate::localization::TzLocation::from_coords(coords);
        Self { holidays, locale, approx_bound_interval_size: None }
    }
}

impl Default for Context<NoLocation> {
    fn default() -> Self {
        Self {
            holidays: Default::default(),
            locale: NoLocation,
            approx_bound_interval_size: None,
        }
    }
}
