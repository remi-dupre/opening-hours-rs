use chrono::{Datelike, NaiveDate};
use opening_hours_syntax::rules::time::TimeEvent;
use sunrise_next::{DawnType, SolarDay, SolarEvent};

/// A valid pair of geographic coordinates.
///
/// See https://en.wikipedia.org/wiki/Geographic_coordinate_system
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Coordinates {
    lat: f64,
    lon: f64,
}

impl Coordinates {
    /// Validate a pair of latitude / longitude.
    ///
    /// Return `None` if values are out of range (`abs(lat) > 90` or
    /// `abs(lon) > 180`).
    pub const fn new(lat: f64, lon: f64) -> Option<Self> {
        if lat.is_nan() || lon.is_nan() || lat < -90.0 || lat > 90.0 || lon < -180.0 || lon > 180.0
        {
            return None;
        }

        Some(Self { lat, lon })
    }

    /// Get the time for a sun event at a given date.
    pub fn event_time(&self, date: NaiveDate, event: TimeEvent) -> chrono::DateTime<chrono::Utc> {
        let solar_event = match event {
            TimeEvent::Dawn => SolarEvent::Dawn(DawnType::Civil),
            TimeEvent::Sunrise => SolarEvent::Sunrise,
            TimeEvent::Sunset => SolarEvent::Sunset,
            TimeEvent::Dusk => SolarEvent::Dusk(DawnType::Civil),
        };

        let solar_day = SolarDay::new(self.lat, self.lon, date.year(), date.month(), date.day());
        let timestamp = solar_day.event_time(solar_event);
        chrono::DateTime::from_timestamp(timestamp, 0).expect("invalid timestamp")
    }

    /// Get latitude component.
    pub fn lat(&self) -> f64 {
        self.lat
    }

    /// Get longitude component.
    pub fn lon(&self) -> f64 {
        self.lon
    }
}

impl std::fmt::Display for Coordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.lat, self.lon)
    }
}
