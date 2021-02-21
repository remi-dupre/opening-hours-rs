use std::ops::Range;

use chrono::{Duration, NaiveDate};

use crate::utils::{range_intersection, time_ranges_union};
use opening_hours_syntax::extended_time::ExtendedTime;

#[derive(Clone, Debug)]
pub struct TimeSelector {
    pub time: Vec<TimeSpan>,
}

impl TimeSelector {
    pub fn new(time: Vec<TimeSpan>) -> Self {
        if time.is_empty() {
            Self::default()
        } else {
            Self { time }
        }
    }

    pub fn intervals_at(&self, date: NaiveDate) -> impl Iterator<Item = Range<ExtendedTime>> + '_ {
        time_ranges_union(self.as_naive_time(date).filter_map(|range| {
            let dstart = ExtendedTime::new(0, 0);
            let dend = ExtendedTime::new(24, 0);
            range_intersection(range, dstart..dend)
        }))
    }

    pub fn intervals_at_next_day(
        &self,
        date: NaiveDate,
    ) -> impl Iterator<Item = Range<ExtendedTime>> + '_ {
        time_ranges_union(
            self.as_naive_time(date)
                .filter_map(|range| {
                    let dstart = ExtendedTime::new(24, 0);
                    let dend = ExtendedTime::new(48, 0);
                    range_intersection(range, dstart..dend)
                })
                .map(|range| {
                    let start = range.start.add_hours(-24).unwrap();
                    let end = range.end.add_hours(-24).unwrap();
                    start..end
                }),
        )
    }

    pub fn as_naive_time(&self, date: NaiveDate) -> impl Iterator<Item = Range<ExtendedTime>> + '_ {
        self.time.iter().map(move |span| span.as_naive_time(date))
    }
}

impl Default for TimeSelector {
    fn default() -> Self {
        Self {
            time: vec![TimeSpan::fixed_range(
                ExtendedTime::new(0, 0),
                ExtendedTime::new(24, 0),
            )],
        }
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
    pub fn fixed_range(start: ExtendedTime, end: ExtendedTime) -> Self {
        Self {
            range: Time::Fixed(start)..Time::Fixed(end),
            open_end: false,
            repeats: None,
        }
    }

    pub fn as_naive_time(&self, date: NaiveDate) -> Range<ExtendedTime> {
        let start = self.range.start.as_naive(date);
        let end = self.range.end.as_naive(date);

        // If end < start, it actually wraps to next day
        let end = {
            if start <= end {
                end
            } else {
                end.add_hours(24)
                    .expect("overflow during TimeSpan resolution")
            }
        };

        assert!(start <= end);
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
            Self::Dusk => ExtendedTime::new(20, 0),
        }
    }
}
