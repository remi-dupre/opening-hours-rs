use std::fmt::Display;
use std::ops::Range;

use chrono::Duration;

use crate::display::write_selector;
use crate::extended_time::ExtendedTime;

// TimeSelector

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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
                ExtendedTime::new(0, 0).unwrap(),
                ExtendedTime::new(24, 0).unwrap(),
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

impl Display for TimeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.range.start)?;

        if self.range.start != self.range.end {
            write!(f, "-{}", self.range.end)?;
        }

        if self.open_end {
            write!(f, "+")?;
        }

        if let Some(repeat) = self.repeats {
            if repeat.num_hours() > 0 {
                write!(f, "{:02}:", repeat.num_hours())?;
            }

            write!(f, "{:02}", repeat.num_minutes() % 60)?;
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

        if self.offset > 0 {
            write!(f, "+{}", self.offset)?;
        } else {
            write!(f, "{}", self.offset)?;
        }

        Ok(())
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
