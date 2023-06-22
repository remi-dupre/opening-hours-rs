use std::marker::PhantomData;

use chrono::{DateTime, NaiveDateTime};
use chrono::{Duration, TimeZone};

#[cfg(feature = "tracing")]
use tracing::warn;

use crate::error::Result;

// TODO: doc
pub trait Localize: Clone {
    type Output;
    type WithTz<T: TimeZone>: Localize;
    type WithCoordInferTz: Localize;

    fn localize_datetime(&self, naive: NaiveDateTime) -> Self::Output;

    #[cfg(feature = "localize")]
    fn with_tz<T: TimeZone>(self, tz: T) -> Self::WithTz<T>;

    #[cfg(feature = "localize")]
    fn try_with_coord_infer_tz(self, lat: f64, lon: f64) -> Result<Self::WithCoordInferTz>;
}

pub trait LocalizeWithTz: Localize {
    type WithCoord: Localize;

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
    type Output = NaiveDateTime;
    type WithTz<T: TimeZone> = WithTz<T>;
    type WithCoordInferTz = WithCoordAndTz<chrono_tz::Tz>;

    fn localize_datetime(&self, naive: NaiveDateTime) -> Self::Output {
        naive
    }

    #[cfg(feature = "localize")]
    fn with_tz<T: TimeZone>(self, tz: T) -> Self::WithTz<T> {
        WithTz { tz }
    }

    #[cfg(feature = "localize")]
    fn try_with_coord_infer_tz(self, lat: f64, lon: f64) -> Result<Self::WithCoordInferTz> {
        todo!()
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
    type Output = DateTime<Tz>;
    type WithTz<N: TimeZone> = WithTz<N>;
    type WithCoordInferTz = WithCoordAndTz<Tz>;

    fn localize_datetime(&self, naive: NaiveDateTime) -> Self::Output {
        localize_next_valid(naive, &self.tz)
    }

    #[cfg(feature = "localize")]
    fn with_tz<N: TimeZone>(self, tz: N) -> Self::WithTz<N> {
        WithTz { tz }
    }

    #[cfg(feature = "localize")]
    fn try_with_coord_infer_tz(self, lat: f64, lon: f64) -> Result<Self::WithCoordInferTz> {
        todo!()
    }
}

#[cfg(feature = "localize")]
impl<Tz: TimeZone> LocalizeWithTz for WithTz<Tz> {
    type WithCoord = WithCoordAndTz<Tz>;

    fn with_coord(self, lat: f64, lon: f64) -> Self::WithCoordInferTz {
        WithCoordAndTz { tz: self.tz, lat, lon }
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
    type Output = DateTime<Tz>;
    type WithTz<N: TimeZone> = WithCoordAndTz<N>;
    type WithCoordInferTz = WithCoordAndTz<Tz>;

    fn localize_datetime(&self, naive: NaiveDateTime) -> Self::Output {
        localize_next_valid(naive, &self.tz)
    }

    #[cfg(feature = "localize")]
    fn with_tz<T: TimeZone>(self, tz: T) -> Self::WithTz<T> {
        todo!()
    }

    #[cfg(feature = "localize")]
    fn try_with_coord_infer_tz(self, lat: f64, lon: f64) -> Result<Self> {
        todo!()
    }
}

#[cfg(feature = "localize")]
impl<Tz: TimeZone> LocalizeWithTz for WithCoordAndTz<Tz> {
    type WithCoord = WithCoordAndTz<Tz>;

    fn with_coord(self, lat: f64, lon: f64) -> Self::WithCoordInferTz {
        todo!()
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
