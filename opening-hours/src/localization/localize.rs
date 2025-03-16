use std::fmt::Debug;
use std::ops::Add;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone};
use opening_hours_syntax::rules::time::TimeEvent;

use crate::localization::Coordinates;

/// Specifies how dates should be localized while evaluating opening hours. No
/// localisation is available by default but this can be used to specify a
/// timezone and coordinates (which affect sun events).
pub trait Localize: Clone + Send + Sync + 'static {
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
    coords: Option<Coordinates>,
}

impl<Tz> TzLocation<Tz>
where
    Tz: TimeZone + Send + Sync,
{
    /// Create a new location context which only contains timezone information.
    pub fn new(tz: Tz) -> Self {
        Self { tz, coords: None }
    }

    /// Extract the timezone for this location.
    pub fn get_timezone(&self) -> &Tz {
        &self.tz
    }

    /// Attach coordinates to the location context.
    ///
    /// If coordinates where already specified, they will be replaced with the
    /// new ones.
    pub fn with_coords(self, coords: Coordinates) -> Self {
        Self { tz: self.tz, coords: Some(coords) }
    }
}

#[cfg(feature = "auto-timezone")]
impl TzLocation<chrono_tz::Tz> {
    /// Create a new location context from a set of coordinates and with timezone
    /// information inferred from this localization.
    ///
    /// Returns `None` if latitude or longitude is invalid.
    ///
    /// ```
    /// use chrono_tz::Europe;
    /// use opening_hours::localization::{Coordinates, TzLocation};
    ///
    /// let coords = Coordinates::new(48.8535, 2.34839).unwrap();
    ///
    /// assert_eq!(
    ///     TzLocation::from_coords(coords),
    ///     TzLocation::new(Europe::Paris).with_coords(coords),
    /// );
    /// ```
    pub fn from_coords(coords: Coordinates) -> Self {
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

        let tz_name = TZ_NAME_FINDER.get_tz_name(coords.lon(), coords.lat());

        #[allow(clippy::unnecessary_lazy_evaluations)]
        let tz = TZ_BY_NAME.get(tz_name).copied().unwrap_or_else(|| {
            #[cfg(feature = "log")]
            log::warn!("Could not find time zone `{tz_name}` at {coords}");
            chrono_tz::UTC
        });

        Self::new(tz).with_coords(coords)
    }
}

impl<Tz> Localize for TzLocation<Tz>
where
    Tz: TimeZone + Send + Sync + 'static,
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
        let Some(coords) = self.coords else {
            return NoLocation.event_time(date, event);
        };

        let dt = coords.event_time(date, event).with_timezone(&self.tz);
        self.naive(dt).time()
    }
}
