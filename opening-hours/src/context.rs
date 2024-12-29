use std::fmt::Debug;
use std::ops::Add;
use std::sync::Arc;

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone};
use compact_calendar::CompactCalendar;
use opening_hours_syntax::rules::time::TimeEvent;
use sunrise_next::{DawnType, SolarDay, SolarEvent};

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

// --
// -- Localization
// --

/// Specifies how dates should be localized while evaluating opening hours. No
/// localisation is available by default but this can be used to specify a
/// timezone and coordinates (which affect sun events).
pub trait Localize: Clone + Send + Sync {
    /// The type for localized date & time.
    type DateTime: Clone + Add<Duration, Output = Self::DateTime>;

    /// Get naive local time.
    fn naive(&self, dt: Self::DateTime) -> NaiveDateTime;

    /// Localize a naive datetime.
    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime;

    /// Get the localized time for a sun event at a given date.
    fn event_time(&self, _date: NaiveDate, event: TimeEvent) -> NaiveTime {
        match event {
            TimeEvent::Dawn => NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            TimeEvent::Sunrise => NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            TimeEvent::Sunset => NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
            TimeEvent::Dusk => NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
        }
    }
}

// No location info.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct NoLocation;

impl Localize for NoLocation {
    type DateTime = NaiveDateTime;

    fn naive(&self, dt: Self::DateTime) -> NaiveDateTime {
        dt
    }

    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime {
        naive
    }
}

/// Time zone is specified and coordinates can optionally be specified for
/// accurate sun events.
#[derive(Clone, Debug, PartialEq)]
pub struct TzLocation<Tz>
where
    Tz: TimeZone + Send + Sync,
{
    tz: Tz,
    coords: Option<[f64; 2]>,
}

impl<Tz> TzLocation<Tz>
where
    Tz: TimeZone + Send + Sync,
{
    /// Create a new location context which only contains timezone information.
    pub fn new(tz: Tz) -> Self {
        Self { tz, coords: None }
    }

    /// Attach coordinates to the location context.
    ///
    /// If coordinates where already specified, they will be replaced with the
    /// new ones.
    pub fn with_coords(self, lat: f64, lon: f64) -> Self {
        Self { tz: self.tz, coords: Some([lat, lon]) }
    }
}

#[cfg(feature = "auto-timezone")]
impl TzLocation<chrono_tz::Tz> {
    /// Create a new location context from a set of coordinates and with timezone
    /// information inferred from this localization.
    ///
    /// ```
    /// use chrono_tz::Europe;
    /// use opening_hours::TzLocation;
    ///
    /// assert_eq!(
    ///     TzLocation::from_coords(48.8535, 2.34839),
    ///     TzLocation::new(Europe::Paris).with_coords(48.8535, 2.34839),
    /// );
    /// ```
    pub fn from_coords(lat: f64, lon: f64) -> Self {
        use std::collections::HashMap;
        use std::sync::LazyLock;

        static TZ_NAME_FINDER: LazyLock<tzf_rs::DefaultFinder> =
            LazyLock::new(tzf_rs::DefaultFinder::new);

        static TZ_BY_NAME: LazyLock<HashMap<&str, chrono_tz::Tz>> = LazyLock::new(|| {
            chrono_tz::TZ_VARIANTS
                .iter()
                .copied()
                .map(|tz| (tz.name(), tz))
                .collect()
        });

        let tz_name = TZ_NAME_FINDER.get_tz_name(lon, lat);

        #[allow(clippy::unnecessary_lazy_evaluations)]
        let tz = TZ_BY_NAME.get(tz_name).copied().unwrap_or_else(|| {
            #[cfg(feature = "log")]
            log::warn!("Could not find time zone `{tz_name}` at {lat},{lon}");
            chrono_tz::UTC
        });

        Self { tz, coords: Some([lat, lon]) }
    }
}

impl<Tz> Localize for TzLocation<Tz>
where
    Tz: TimeZone + Send + Sync,
    Tz::Offset: Send + Sync,
{
    type DateTime = chrono::DateTime<Tz>;

    fn naive(&self, dt: Self::DateTime) -> NaiveDateTime {
        dt.with_timezone(&self.tz).naive_local()
    }

    fn datetime(&self, mut naive: NaiveDateTime) -> Self::DateTime {
        loop {
            if let Some(dt) = self.tz.from_local_datetime(&naive).latest() {
                return dt;
            }

            naive = naive
                .checked_add_signed(TimeDelta::minutes(1))
                .expect("no valid datetime for time zone");
        }
    }

    fn event_time(&self, date: NaiveDate, event: TimeEvent) -> NaiveTime {
        let Some([lat, lon]) = self.coords else {
            return NoLocation.event_time(date, event);
        };

        let solar_event = match event {
            TimeEvent::Dawn => SolarEvent::Dawn(DawnType::Civil),
            TimeEvent::Sunrise => SolarEvent::Sunrise,
            TimeEvent::Sunset => SolarEvent::Sunset,
            TimeEvent::Dusk => SolarEvent::Dusk(DawnType::Civil),
        };

        let solar_day = SolarDay::new(lat, lon, date.year(), date.month(), date.day());
        let timestamp = solar_day.event_time(solar_event);

        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp, 0)
            .expect("invalid timestamp")
            .with_timezone(&self.tz);

        self.naive(dt).time()
    }
}

// --
// -- Context
// --

/// All the context attached to a parsed OpeningHours expression and that can
/// alter its evaluation semantics.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Context<L = NoLocation> {
    pub holidays: ContextHolidays,
    pub locale: L,
}

impl<L> Context<L> {
    /// Attach a new holidays component to this context.
    pub fn with_holidays(self, holidays: ContextHolidays) -> Self {
        Self { holidays, ..self }
    }

    /// Attach a new locale component to this context.
    pub fn with_locale<L2: Localize>(self, locale: L2) -> Context<L2> {
        Context { holidays: self.holidays, locale }
    }
}

#[cfg(feature = "auto-timezone")]
impl Context<TzLocation<chrono_tz::Tz>> {
    /// Create a context with given coordinates and try to infer a timezone and
    /// a local holiday calendar.
    ///
    /// ```
    /// use opening_hours::{Context, TzLocation};
    /// use opening_hours::country::Country;
    ///
    /// assert_eq!(
    ///     Context::from_coords(48.8535, 2.34839),
    ///     Context::default()
    ///         .with_holidays(Country::FR.holidays())
    ///         .with_locale(TzLocation::from_coords(48.8535, 2.34839)),
    /// );
    /// ```
    #[cfg(feature = "auto-country")]
    pub fn from_coords(lat: f64, lon: f64) -> Self {
        use crate::country::Country;

        let holidays = Country::try_from_coords(lat, lon)
            .map(Country::holidays)
            .unwrap_or_default();

        let locale = TzLocation::from_coords(lat, lon);
        Self { holidays, locale }
    }
}

impl Default for Context<NoLocation> {
    fn default() -> Self {
        Self { holidays: Default::default(), locale: NoLocation }
    }
}
