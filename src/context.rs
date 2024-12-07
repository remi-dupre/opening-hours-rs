use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Add;
use std::sync::{Arc, LazyLock};

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
    /// The type for localized date & time.
    type DateTime: Clone + Add<Duration, Output = Self::DateTime>;

    /// The type that results in specifying a new time zone.
    type WithTz<T>: LocalizeWithTz
    where
        T: TimeZone + Send + Sync,
        T::Offset: Send + Sync;

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

    /// Specify a new time zone
    fn with_tz<T>(self, tz: T) -> Self::WithTz<T>
    where
        T: TimeZone + Send + Sync,
        T::Offset: Send + Sync;

    /// Automatically infer timezone from input coordinates.
    fn with_tz_from_coords(
        self,
        lat: f64,
        lon: f64,
    ) -> <Self::WithTz<chrono_tz::Tz> as LocalizeWithTz>::WithCoord {
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

        let tz = TZ_BY_NAME.get(tz_name).copied().unwrap_or_else(|| {
            log::warn!("Could not find time zone `{tz_name}` at {lat},{lon}");
            chrono_tz::UTC
        });

        self.with_tz(tz).with_coord(lat, lon)
    }
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
        Tz: TimeZone + Send + Sync,
        Tz::Offset: Send + Sync;

    fn naive(&self, dt: Self::DateTime) -> NaiveDateTime {
        dt
    }

    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime {
        naive
    }

    fn with_tz<Tz>(self, tz: Tz) -> Self::WithTz<Tz>
    where
        Tz: TimeZone + Send + Sync,
        Tz::Offset: Send + Sync,
    {
        TzLocation { tz }
    }
}

/// Time zone is specified.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct TzLocation<Tz>
where
    Tz: TimeZone + Send + Sync,
{
    tz: Tz,
}

impl<Tz> Localize for TzLocation<Tz>
where
    Tz: TimeZone + Send + Sync,
    Tz::Offset: Send + Sync,
{
    type DateTime = chrono::DateTime<Tz>;

    type WithTz<T>
        = TzLocation<T>
    where
        T: TimeZone + Send + Sync,
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
        T: TimeZone + Send + Sync,
        T::Offset: Send + Sync,
    {
        TzLocation { tz }
    }
}

impl<Tz> LocalizeWithTz for TzLocation<Tz>
where
    Tz: TimeZone + Send + Sync,
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
    Tz: TimeZone + Send + Sync,
{
    tz: Tz,
    lat: f64,
    lon: f64,
}

impl<Tz> Localize for CoordLocation<Tz>
where
    Tz: TimeZone + Send + Sync,
    Tz::Offset: Send + Sync,
{
    type DateTime = chrono::DateTime<Tz>;

    type WithTz<T>
        = CoordLocation<T>
    where
        T: TimeZone + Send + Sync,
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

    fn event_time(&self, date: NaiveDate, event: TimeEvent) -> NaiveTime {
        let solar_event = match event {
            TimeEvent::Dawn => SolarEvent::Dawn(DawnType::Civil),
            TimeEvent::Sunrise => SolarEvent::Sunrise,
            TimeEvent::Sunset => SolarEvent::Sunset,
            TimeEvent::Dusk => SolarEvent::Dusk(DawnType::Civil),
        };

        let solar_day = SolarDay::new(self.lat, self.lon, date.year(), date.month(), date.day());
        let timestamp = solar_day.event_time(dbg!(solar_event));

        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp, 0)
            .expect("invalid timestamp")
            .with_timezone(&self.tz);

        self.naive(dt).time()
    }

    fn with_tz<T>(self, tz: T) -> Self::WithTz<T>
    where
        T: TimeZone + Send + Sync,
        T::Offset: Send + Sync,
    {
        CoordLocation { tz, lat: self.lat, lon: self.lon }
    }
}

impl<Tz> LocalizeWithTz for CoordLocation<Tz>
where
    Tz: TimeZone + Send + Sync,
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
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Context<L = NoLocation> {
    pub(crate) holidays: ContextHolidays,
    pub(crate) locale: L,
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

impl Default for Context<NoLocation> {
    fn default() -> Self {
        Self {
            holidays: Default::default(),
            locale: Default::default(),
        }
    }
}
