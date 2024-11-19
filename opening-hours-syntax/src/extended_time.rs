use std::convert::TryInto;
use std::fmt::{Debug, Display};

use chrono::{NaiveTime, Timelike};

/// An hour+minute struct that can go up to 48h.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExtendedTime {
    hour: u8,
    minute: u8,
}

impl ExtendedTime {
    /// Create a new extended time, this may return `None` if input values are
    /// out of range.
    ///
    /// ```
    /// use opening_hours_syntax::ExtendedTime;
    ///
    /// assert!(ExtendedTime::new(28, 30).is_some());
    /// assert!(ExtendedTime::new(72, 15).is_none()); // hours are out of bound
    /// assert!(ExtendedTime::new(24, 60).is_none()); // minutes are out of bound
    /// ```
    #[inline]
    pub const fn new(hour: u8, minute: u8) -> Option<Self> {
        if hour > 48 || minute > 59 || (hour == 48 && minute > 0) {
            None
        } else {
            Some(Self { hour, minute })
        }
    }

    /// Get the number of full hours in this extended time.
    ///
    /// ```
    /// use opening_hours_syntax::ExtendedTime;
    ///
    /// let time = ExtendedTime::new(27, 35).unwrap();
    /// assert_eq!(time.hour(), 27);
    /// ```
    #[inline]
    pub fn hour(self) -> u8 {
        self.hour
    }

    /// Get the number of remaining minutes in this extended time.
    ///
    /// ```
    /// use opening_hours_syntax::ExtendedTime;
    ///
    /// let time = ExtendedTime::new(27, 35).unwrap();
    /// assert_eq!(time.minute(), 35);
    /// ```
    #[inline]
    pub fn minute(self) -> u8 {
        self.minute
    }

    /// Add plain minutes to the extended time and return `None` if this
    /// results in it being out of bounds.
    ///
    /// ```
    /// use opening_hours_syntax::ExtendedTime;
    ///
    /// let time = ExtendedTime::new(24, 0).unwrap();
    /// assert_eq!(time.add_minutes(75), ExtendedTime::new(25, 15));
    /// assert!(time.add_minutes(24 * 60 + 1).is_none());
    /// assert!(time.add_minutes(-24 * 60 - 1).is_none());
    /// ```
    #[inline]
    pub fn add_minutes(self, minutes: i16) -> Option<Self> {
        let as_minutes = (self.mins_from_midnight() as i16).checked_add(minutes)?;
        Self::from_mins_from_midnight(as_minutes.try_into().ok()?)
    }

    /// Add plain hours to the extended time and return `None` if this results
    /// in it being out of bounds.
    ///
    /// ```
    /// use opening_hours_syntax::ExtendedTime;
    ///
    /// let time = ExtendedTime::new(24, 15).unwrap();
    /// assert_eq!(time.add_hours(3), ExtendedTime::new(27, 15));
    /// assert!(time.add_hours(25).is_none());
    /// assert!(time.add_hours(-25).is_none());
    /// ```
    #[inline]
    pub fn add_hours(self, hours: i8) -> Option<Self> {
        Self::new(
            (i16::from(self.hour) + i16::from(hours)).try_into().ok()?,
            self.minute,
        )
    }

    /// Get the total number of minutes from *00:00*.
    ///
    /// ```
    /// use opening_hours_syntax::ExtendedTime;
    ///
    /// let time = ExtendedTime::new(25, 15).unwrap();
    /// assert_eq!(time.mins_from_midnight(), 25 * 60 + 15);
    /// ```
    #[inline]
    pub fn mins_from_midnight(self) -> u16 {
        u16::from(self.minute) + 60 * u16::from(self.hour)
    }

    /// Build an extended time from the total number of minutes from midnight
    /// and return `None` if the result is out of bounds.
    ///
    /// ```
    /// use opening_hours_syntax::ExtendedTime;
    ///
    /// assert_eq!(
    ///     ExtendedTime::from_mins_from_midnight(26 * 60 + 15),
    ///     ExtendedTime::new(26, 15),
    /// );
    ///
    /// assert!(ExtendedTime::from_mins_from_midnight(65_000).is_none());
    /// ```
    #[inline]
    pub fn from_mins_from_midnight(minute: u16) -> Option<Self> {
        let hour = (minute / 60).try_into().ok()?;
        let minute = (minute % 60).try_into().ok()?;
        Self::new(hour, minute)
    }
}

impl Display for ExtendedTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

impl Debug for ExtendedTime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{self}")
    }
}

impl TryInto<NaiveTime> for ExtendedTime {
    type Error = ();

    #[inline]
    fn try_into(self) -> Result<NaiveTime, Self::Error> {
        NaiveTime::from_hms_opt(self.hour.into(), self.minute.into(), 0).ok_or(())
    }
}

impl From<NaiveTime> for ExtendedTime {
    #[inline]
    fn from(time: NaiveTime) -> ExtendedTime {
        Self {
            hour: time.hour().try_into().expect("invalid NaiveTime"),
            minute: time.minute().try_into().expect("invalid NaiveTime"),
        }
    }
}
