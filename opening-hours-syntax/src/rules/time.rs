use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::{Range, RangeInclusive};

use chrono::Duration;

use crate::display::write_selector;
use crate::extended_time::ExtendedTime;

// TimeSelector

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TimeSelector {
    pub time: Vec<TimeSpan>,
}

impl TimeSelector {
    /// Note that not all cases can be covered
    pub(crate) fn is_00_24(&self) -> bool {
        self.time.len() == 1
            && self.time.first()
                == Some(&TimeSpan::fixed_range(
                    ExtendedTime::MIDNIGHT_00,
                    ExtendedTime::MIDNIGHT_24,
                ))
    }
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
                ExtendedTime::MIDNIGHT_00,
                ExtendedTime::MIDNIGHT_24,
            )],
        }
    }
}

impl Display for TimeSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_selector(f, &self.time)
    }
}

// TimeSpan

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TimeSpan {
    Range {
        range: Range<Time>,
        open_end: bool,
    },
    Repeat {
        range: RangeInclusive<Time>,
        repeats: Duration,
    },
}

impl TimeSpan {
    #[inline]
    pub const fn fixed_range(start: ExtendedTime, end: ExtendedTime) -> Self {
        Self::Range {
            range: Time::Fixed(start)..Time::Fixed(end),
            open_end: false,
        }
    }

    pub fn start(&self) -> Time {
        match self {
            Self::Range { range, open_end: _ } => range.start,
            Self::Repeat { range, repeats: _ } => *range.start(),
        }
    }

    pub fn end(&self) -> Time {
        match self {
            Self::Range { range, open_end: _ } => range.end,
            Self::Repeat { range, repeats: _ } => *range.end(),
        }
    }

    fn open_end(&self) -> bool {
        match self {
            Self::Range { range: _, open_end } => *open_end,
            Self::Repeat { .. } => false,
        }
    }
}

impl Display for TimeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.start())?;

        if self.start() != self.end()
            && !(self.open_end() && self.end() == Time::Fixed(ExtendedTime::MIDNIGHT_24))
        {
            write!(f, "-{}", self.end())?;
        }

        if self.open_end() {
            write!(f, "+")?;
        }

        if let Self::Repeat { range: _, repeats } = self {
            write!(f, "/")?;

            if repeats.num_hours() > 0 {
                write!(f, "{:02}:", repeats.num_hours())?;
            }

            write!(f, "{:02}", repeats.num_minutes() % 60)?;
        }

        Ok(())
    }
}

// Time

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Time {
    Fixed(ExtendedTime),
    Variable(VariableTime),
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed(time) => write!(f, "{time}"),
            Self::Variable(time) => write!(f, "{time}"),
        }
    }
}

// VariableTime

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct VariableTime {
    pub event: TimeEvent,
    pub offset: i16,
}

impl Display for VariableTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.event)?;

        match self.offset.cmp(&0) {
            Ordering::Less => write!(f, "{}", self.offset),
            Ordering::Greater => write!(f, "+{}", self.offset),
            Ordering::Equal => Ok(()),
        }
    }
}

// TimeEvent

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum TimeEvent {
    Dawn,
    Sunrise,
    Sunset,
    Dusk,
}

impl TimeEvent {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Dawn => "dawn",
            Self::Sunrise => "sunrise",
            Self::Sunset => "sunset",
            Self::Dusk => "dusk",
        }
    }
}

impl Display for TimeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
