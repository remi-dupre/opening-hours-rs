use chrono::NaiveDate;
use opening_hours_syntax::rules::time::TimeEvent;
use sunrise::{DawnType, SolarDay, SolarEvent};

/// A valid pair of geographic coordinates.
///
/// See https://en.wikipedia.org/wiki/Geographic_coordinate_system
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Coordinates(sunrise::Coordinates);

impl Coordinates {
    /// Validate a pair of latitude / longitude.
    ///
    /// Return `None` if values are out of range (`abs(lat) > 90` or
    /// `abs(lon) > 180`).
    pub const fn new(lat: f64, lon: f64) -> Option<Self> {
        match sunrise::Coordinates::new(lat, lon) {
            Some(c) => Some(Self(c)),
            None => None,
        }
    }

    /// Get the time for a sun event at a given date.
    pub fn event_time(&self, date: NaiveDate, event: TimeEvent) -> chrono::DateTime<chrono::Utc> {
        let solar_event = match event {
            TimeEvent::Dawn => SolarEvent::Dawn(DawnType::Civil),
            TimeEvent::Sunrise => SolarEvent::Sunrise,
            TimeEvent::Sunset => SolarEvent::Sunset,
            TimeEvent::Dusk => SolarEvent::Dusk(DawnType::Civil),
        };

        let solar_day = SolarDay::new(self.0, date);
        solar_day.event_time(solar_event)
    }

    /// Get latitude component.
    pub fn lat(&self) -> f64 {
        self.0.lat()
    }

    /// Get longitude component.
    pub fn lon(&self) -> f64 {
        self.0.lon()
    }
}

impl std::fmt::Display for Coordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.lat(), self.lon())
    }
}
