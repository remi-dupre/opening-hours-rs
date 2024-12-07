use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Add;
use std::sync::Arc;

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Timelike};
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
        + Sync
        + Send
        + Datelike
        + Timelike
        + Add<Duration, Output = Self::DateTime>;

    /// The type that results in specifying a new time zone
    type WithTz<T>: LocalizeWithTz
    where
        T: chrono::TimeZone + Send + Sync,
        T::Offset: Send + Sync;

    /// Get naive local time
    fn naive(&self, dt: Self::DateTime) -> NaiveDateTime;

    /// Localize a naive datetime
    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime;

    /// Get the localized time for a sun event at a given date
    fn event_time(&self, _date: NaiveDate, event: TimeEvent) -> NaiveTime {
        match event {
            TimeEvent::Dawn => NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            TimeEvent::Sunrise => NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            TimeEvent::Sunset => NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
            TimeEvent::Dusk => NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
        }
    }

    /// Specify a new time zone
    fn with_tz<T>(self, tz: T) -> Self::WithTz<T>
    where
        T: chrono::TimeZone + Send + Sync,
        T::Offset: Send + Sync;

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

    type WithTz<Tz>
        = TzLocation<Tz>
    where
        Tz: chrono::TimeZone + Send + Sync,
        Tz::Offset: Send + Sync;

    fn naive(&self, dt: Self::DateTime) -> NaiveDateTime {
        dt
    }

    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime {
        naive
    }

    fn with_tz<Tz>(self, tz: Tz) -> Self::WithTz<Tz>
    where
        Tz: chrono::TimeZone + Send + Sync,
        Tz::Offset: Send + Sync,
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
    Tz::Offset: Send + Sync,
{
    type DateTime = chrono::DateTime<Tz>;

    type WithTz<T>
        = TzLocation<T>
    where
        T: chrono::TimeZone + Send + Sync,
        T::Offset: Send + Sync;

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

    fn with_tz<T>(self, tz: T) -> Self::WithTz<T>
    where
        T: chrono::TimeZone + Send + Sync,
        T::Offset: Send + Sync,
    {
        TzLocation { tz }
    }
}

impl<Tz> LocalizeWithTz for TzLocation<Tz>
where
    Tz: chrono::TimeZone + Send + Sync,
    Tz::Offset: Send + Sync,
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
    Tz::Offset: Send + Sync,
{
    type DateTime = chrono::DateTime<Tz>;

    type WithTz<T>
        = CoordLocation<T>
    where
        T: chrono::TimeZone + Send + Sync,
        T::Offset: Send + Sync;

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

    fn with_tz<T>(self, tz: T) -> Self::WithTz<T>
    where
        T: chrono::TimeZone + Send + Sync,
        T::Offset: Send + Sync,
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
    Tz::Offset: Send + Sync,
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
