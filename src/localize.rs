use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Add;

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use chrono::{Duration, TimeZone};
use once_cell::sync::Lazy;
use sunrise::{DawnType, SolarDay, SolarEvent};

#[cfg(feature = "tracing")]
use tracing::warn;

use crate::error::{Error, Result};
use opening_hours_syntax::rules::time::TimeEvent;

#[cfg(feature = "localize")]
static TZ_NAME_FINDER: Lazy<tzf_rs::DefaultFinder> = Lazy::new(tzf_rs::DefaultFinder::new);

#[cfg(feature = "localize")]
static TZ_BY_NAME: Lazy<HashMap<&str, chrono_tz::Tz>> = Lazy::new(|| {
    chrono_tz::TZ_VARIANTS
        .iter()
        .copied()
        .map(|tz| (tz.name(), tz))
        .collect()
});

// TODO: doc
pub trait Localize: Clone {
    type DateTime: Clone
        + Debug
        + Eq
        + Ord
        + Datelike
        + Timelike
        + Add<Duration, Output = Self::DateTime>;

    type WithTz<T: TimeZone>: LocalizeWithTz;
    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime;

    fn event_time(&self, _date: NaiveDate, event: TimeEvent) -> NaiveTime {
        match event {
            TimeEvent::Dawn => NaiveTime::from_hms_opt(6, 0, 0),
            TimeEvent::Sunrise => NaiveTime::from_hms_opt(7, 0, 0),
            TimeEvent::Sunset => NaiveTime::from_hms_opt(19, 0, 0),
            TimeEvent::Dusk => NaiveTime::from_hms_opt(20, 0, 0),
        }
        .unwrap()
    }

    #[cfg(feature = "localize")]
    fn with_tz<T: TimeZone>(self, tz: T) -> Self::WithTz<T>;

    #[cfg(feature = "localize")]
    fn try_with_coord_infer_tz(
        self,
        lat: f64,
        lon: f64,
    ) -> Result<<Self::WithTz<chrono_tz::Tz> as LocalizeWithTz>::WithCoord> {
        let tz_name = TZ_NAME_FINDER.get_tz_name(lon, lat);

        let tz = TZ_BY_NAME
            .get(tz_name)
            .copied()
            .ok_or_else(|| Error::TzNotFound(tz_name))?;

        #[cfg(feature = "tracing")]
        {
            tracing::debug!("TimeZone at ({lat},{lon}) is {tz}");
        }

        Ok(self.with_tz(tz).with_coord(lat, lon))
    }
}

pub trait LocalizeWithTz: Localize {
    type WithCoord: LocalizeWithTz;

    #[cfg(feature = "localize")]
    fn with_coord(self, lat: f64, lon: f64) -> Self::WithCoord;
}

// ---
// --- Localisation with no info
// ---

// No location info.
#[derive(Clone, Debug, Default)]
pub struct NoLocation {
    _private: PhantomData<()>,
}

#[cfg(feature = "localize")]
impl Localize for NoLocation {
    type DateTime = NaiveDateTime;
    type WithTz<T: TimeZone> = WithTz<T>;

    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime {
        naive
    }

    #[cfg(feature = "localize")]
    fn with_tz<T: TimeZone>(self, tz: T) -> Self::WithTz<T> {
        WithTz { tz }
    }
}

// ---
// --- Localisation with a timezone.
// ---

#[cfg(feature = "localize")]
#[derive(Clone, Debug)]
pub struct WithTz<Tz: TimeZone> {
    tz: Tz,
}

#[cfg(feature = "localize")]
impl<Tz: TimeZone> Localize for WithTz<Tz> {
    type DateTime = DateTime<Tz>;
    type WithTz<N: TimeZone> = WithTz<N>;

    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime {
        localize_next_valid(naive, &self.tz)
    }

    #[cfg(feature = "localize")]
    fn with_tz<N: TimeZone>(self, tz: N) -> Self::WithTz<N> {
        WithTz { tz }
    }
}

#[cfg(feature = "localize")]
impl<Tz: TimeZone> LocalizeWithTz for WithTz<Tz> {
    type WithCoord = WithCoordAndTz<Tz>;

    fn with_coord(self, lat: f64, lon: f64) -> Self::WithCoord {
        WithCoordAndTz { lat, lon, tz: self.tz }
    }
}

/// Localisation through timezone and coordinates
#[cfg(feature = "localize")]
#[derive(Clone, Debug)]
pub struct WithCoordAndTz<Tz: TimeZone> {
    lat: f64,
    lon: f64,
    tz: Tz,
}

#[cfg(feature = "localize")]
impl<Tz: TimeZone> Localize for WithCoordAndTz<Tz> {
    type DateTime = DateTime<Tz>;
    type WithTz<N: TimeZone> = WithCoordAndTz<N>;

    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime {
        localize_next_valid(naive, &self.tz)
    }

    fn event_time(&self, date: NaiveDate, event: TimeEvent) -> NaiveTime {
        let solar_event = match event {
            TimeEvent::Dawn => SolarEvent::Dawn(DawnType::Civil),
            TimeEvent::Sunrise => SolarEvent::Sunrise,
            TimeEvent::Sunset => SolarEvent::Sunset,
            TimeEvent::Dusk => SolarEvent::Dusk(DawnType::Civil),
        };

        let solar = SolarDay::new(self.lat, self.lon, date.year(), date.month(), date.day());
        let timestamp = solar.event_time(solar_event);

        let datetime = self.tz.from_utc_datetime(
            &NaiveDateTime::from_timestamp_opt(timestamp, 0).expect("invalid timestamp"),
        );

        NaiveTime::from_hms_opt(datetime.hour(), datetime.minute(), datetime.second())
            .expect("invalid local time")
    }

    #[cfg(feature = "localize")]
    fn with_tz<T: TimeZone>(self, tz: T) -> Self::WithTz<T> {
        WithCoordAndTz { lat: self.lat, lon: self.lon, tz }
    }
}

#[cfg(feature = "localize")]
impl<Tz: TimeZone> LocalizeWithTz for WithCoordAndTz<Tz> {
    type WithCoord = WithCoordAndTz<Tz>;

    fn with_coord(self, lat: f64, lon: f64) -> Self::WithCoord {
        Self { lat, lon, tz: self.tz }
    }
}

// TODO: test
/// Localize input datetime to next valid occurence.
#[cfg(feature = "localize")]
fn localize_next_valid<Tz: TimeZone>(naive: NaiveDateTime, tz: &Tz) -> DateTime<Tz> {
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(x) => x,
        chrono::LocalResult::Ambiguous(x, y) => {
            #[cfg(feature = "tracing")]
            {
                warn!("Ambiguous date {naive}: could be {x:?} (default) or {y:?}");
            }

            x
        }
        chrono::LocalResult::None => {
            let mut curr = naive;

            loop {
                curr += Duration::seconds(1);

                if let Some(res) = tz.from_local_datetime(&curr).earliest() {
                    #[cfg(feature = "tracing")]
                    {
                        warn!("Skipped invalid dates from {naive} to {curr}");
                    }

                    return res;
                }
            }
        }
    }
}
