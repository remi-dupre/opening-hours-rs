use std::ops::Range;

use chrono::Duration;

use crate::extended_time::ExtendedTime;

// TimeSelector

#[derive(Clone, Debug)]
pub struct TimeSelector {
    pub time: Vec<TimeSpan>,
}

impl TimeSelector {
    #[inline]
    pub fn new(time: Vec<TimeSpan>) -> Self {
        if time.is_empty() {
            Self::default()
        } else {
            Self { time }
        }
    }
}

impl Default for TimeSelector {
    #[inline]
    fn default() -> Self {
        Self {
            time: vec![TimeSpan::fixed_range(
                ExtendedTime::new(0, 0),
                ExtendedTime::new(24, 0),
            )],
        }
    }
}

// TimeSpan

#[derive(Clone, Debug)]
pub struct TimeSpan {
    pub range: Range<Time>,
    pub open_end: bool,
    pub repeats: Option<Duration>,
}

impl TimeSpan {
    #[inline]
    pub fn fixed_range(start: ExtendedTime, end: ExtendedTime) -> Self {
        Self {
            range: Time::Fixed(start)..Time::Fixed(end),
            open_end: false,
            repeats: None,
        }
    }
}

// Time

#[derive(Copy, Clone, Debug)]
pub enum Time {
    Fixed(ExtendedTime),
    Variable(VariableTime),
}

// VariableTime

#[derive(Copy, Clone, Debug)]
pub struct VariableTime {
    pub event: TimeEvent,
    pub offset: i16,
}

// TimeEvent

#[derive(Clone, Copy, Debug)]
pub enum TimeEvent {
    Dawn,
    Sunrise,
    Sunset,
    Dusk,
}
