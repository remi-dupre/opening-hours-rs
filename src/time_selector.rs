use std::ops::Range;

use chrono::{Duration, NaiveDate};

use crate::extended_time::ExtendedTime;
use crate::utils::time_ranges_union;

#[derive(Clone, Debug, Default)]
pub struct TimeSelector {
    pub time: Vec<TimeSpan>,
}

impl TimeSelector {
    pub fn intervals_at(&self, date: NaiveDate) -> Vec<Range<ExtendedTime>> {
        time_ranges_union(self.time.iter().map(|span| span.as_naive_time(date)))
    }
}

// ---
// --- Time selector
// ---

// TimeSpan

#[derive(Clone, Debug)]
pub struct TimeSpan {
    pub range: Range<Time>,
    pub open_end: bool,
    pub repeats: Option<Duration>,
}

impl TimeSpan {
    pub fn as_naive_time(&self, date: NaiveDate) -> Range<ExtendedTime> {
        let start = self.range.start.as_naive(date);
        let end = self.range.end.as_naive(date);
        start..end
    }
}

// Time

#[derive(Copy, Clone, Debug)]
pub enum Time {
    Fixed(ExtendedTime),
    Variable(VariableTime),
}

impl Time {
    pub fn as_naive(self, date: NaiveDate) -> ExtendedTime {
        match self {
            Time::Fixed(naive) => naive,
            Time::Variable(variable) => variable.as_naive(date),
        }
    }
}

// VariableTime

#[derive(Copy, Clone, Debug)]
pub struct VariableTime {
    pub event: TimeEvent,
    pub offset: i16,
}

impl VariableTime {
    pub fn as_naive(self, date: NaiveDate) -> ExtendedTime {
        self.event
            .as_naive(date)
            .add_minutes(self.offset)
            .unwrap_or_else(|_| ExtendedTime::new(0, 0))
    }
}

// TimeEvent

#[derive(Clone, Copy, Debug)]
pub enum TimeEvent {
    Dawn,
    Sunrise,
    Sunset,
    Dusk,
}

impl TimeEvent {
    pub fn as_naive(self, _date: NaiveDate) -> ExtendedTime {
        // TODO: real computation based on the day (and position/timezone?)
        match self {
            Self::Dawn => ExtendedTime::new(6, 0),
            Self::Sunrise => ExtendedTime::new(7, 0),
            Self::Sunset => ExtendedTime::new(19, 0),
            Self::Dusk => ExtendedTime::new(18, 0),
        }
    }
}
