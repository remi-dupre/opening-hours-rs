use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Add;
use std::sync::Arc;

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use compact_calendar::CompactCalendar;
use opening_hours_syntax::rules::time::TimeEvent;

// --
// -- Holidays
// --

/// Pairs a set of public holidays with a set of school holidays.
#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct ContextHolidays {
    pub public: Arc<CompactCalendar>,
    pub school: Arc<CompactCalendar>,
}

// --
// -- Localization
// --

/// Specifies how dates should be localized while evaluating opening hours. No
/// localisation is available by default but this can be used to specify a
/// TimeZone and coordinates (which affect sun events).
pub trait Localize: Clone + Send + Sync {
    /// The type for localized date & time
    type DateTime: Clone
        + Debug
        + Eq
        + Ord
        + Datelike
        + Timelike
        + Add<Duration, Output = Self::DateTime>;

    /// The type that results in specifying a new time zone
    type WithTz<T: chrono::TimeZone + Send + Sync>: LocalizeWithTz;

    /// Get the localized time for a sun event at a given date
    fn event_time(&self, _date: NaiveDate, event: TimeEvent) -> NaiveTime {
        match event {
            TimeEvent::Dawn => NaiveTime::from_hms_opt(6, 0, 0),
            TimeEvent::Sunrise => NaiveTime::from_hms_opt(7, 0, 0),
            TimeEvent::Sunset => NaiveTime::from_hms_opt(19, 0, 0),
            TimeEvent::Dusk => NaiveTime::from_hms_opt(20, 0, 0),
        }
        .unwrap()
    }

    /// Specify a new tiem zone
    fn with_tz<T: chrono::TimeZone + Send + Sync>(self, tz: T) -> Self::WithTz<T>;

    // // #[cfg(feature = "localize")]
    // fn try_with_coord_infer_tz(
    //     self,
    //     lat: f64,
    //     lon: f64,
    // ) -> crate::error::Result<<Self::WithTz<chrono_tz::Tz> as LocalizeWithTz>::WithCoord> {
    //     let tz_name = TZ_NAME_FINDER.get_tz_name(lon, lat);
    //
    //     let tz = TZ_BY_NAME
    //         .get(tz_name)
    //         .copied()
    //         .ok_or_else(|| crate::error::Error::TzNotFound(tz_name))?;
    //
    //     tracing::debug!("TimeZone at ({lat},{lon}) is {tz}");
    //     Ok(self.with_tz(tz).with_coord(lat, lon))
    // }
}

/// Extend the trait `Localize` for types that can be extended with
/// coordinates. In general, you need to specify a timezone before you set
/// coordinates.
pub trait LocalizeWithTz: Localize {
    /// The type that results in specifying new coordinates
    type WithCoord: LocalizeWithTz;

    /// Specify new coordinates
    fn with_coord(self, lat: f64, lon: f64) -> Self::WithCoord;
}

// No location info.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct NoLocation {
    _private: PhantomData<()>,
}

impl Localize for NoLocation {
    type DateTime = NaiveDateTime;
    type WithTz<Tz: chrono::TimeZone + Send + Sync> = TzLocation<Tz>;

    fn with_tz<Tz>(self, tz: Tz) -> Self::WithTz<Tz>
    where
        Tz: chrono::TimeZone + Send + Sync,
    {
        TzLocation { tz }
    }
}

/// Time zone is specified.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct TzLocation<Tz>
where
    Tz: chrono::TimeZone + Send + Sync,
{
    tz: Tz,
}

impl<Tz> Localize for TzLocation<Tz>
where
    Tz: chrono::TimeZone + Send + Sync,
{
    type DateTime = chrono::DateTime<Tz>;
    type WithTz<T: chrono::TimeZone + Send + Sync> = TzLocation<T>;

    fn with_tz<T: chrono::TimeZone + Send + Sync>(self, tz: T) -> Self::WithTz<T> {
        TzLocation { tz }
    }
}

impl<Tz> LocalizeWithTz for TzLocation<Tz>
where
    Tz: chrono::TimeZone + Send + Sync,
{
    type WithCoord = CoordLocation<Tz>;

    fn with_coord(self, lat: f64, lon: f64) -> Self::WithCoord {
        CoordLocation { tz: self.tz, lat, lon }
    }
}

/// Timezone and coordinates are specified
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoordLocation<Tz>
where
    Tz: chrono::TimeZone + Send + Sync,
{
    tz: Tz,
    lat: f64,
    lon: f64,
}

impl<Tz> Localize for CoordLocation<Tz>
where
    Tz: chrono::TimeZone + Send + Sync,
{
    type DateTime = chrono::DateTime<Tz>;
    type WithTz<T: chrono::TimeZone + Send + Sync> = CoordLocation<T>;

    fn with_tz<T>(self, tz: T) -> Self::WithTz<T>
    where
        T: chrono::TimeZone + Send + Sync,
    {
        CoordLocation { tz, lat: self.lat, lon: self.lon }
    }

    fn event_time(&self, _date: NaiveDate, _event: TimeEvent) -> NaiveTime {
        todo!("Use SolarEvent (maybe behind a feat)")
    }
}

impl<Tz> LocalizeWithTz for CoordLocation<Tz>
where
    Tz: chrono::TimeZone + Send + Sync,
{
    type WithCoord = Self;

    fn with_coord(self, lat: f64, lon: f64) -> Self::WithCoord {
        Self { lat, lon, ..self }
    }
}

// --
// -- Context
// --

/// All the context attached to a parsed OpeningHours expression and that can
/// alter its evaluation semantics.
#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct Context<L: Localize> {
    pub(crate) holidays: ContextHolidays,
    pub(crate) localize: L,
}

impl<L: Localize> Context<L> {
    /// TODO: doc
    pub fn with_holidays(mut self, holidays: ContextHolidays) -> Self {
        self.holidays = holidays;
        self
    }
}
