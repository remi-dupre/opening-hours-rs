use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::Range;

use chrono::Duration;

use crate::display::write_selector;
use crate::extended_time::ExtendedTime;

// TimeSelector

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSelector {
    pub time: Vec<TimeSpan>,
}

impl TimeSelector {
    /// Note that not all cases can be covered
    pub(crate) fn is_00_24(&self) -> bool {
        self.time.len() == 1
            && matches!(
                self.time.first(),
                Some(TimeSpan { range, open_end: false, repeats: None })
                if range.start == Time::Fixed(ExtendedTime::new(0,0).unwrap())
                    && range.end == Time::Fixed(ExtendedTime::new(24,0).unwrap())
            )
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
    pub const fn fixed_range(start: ExtendedTime, end: ExtendedTime) -> Self {
        Self {
            range: Time::Fixed(start)..Time::Fixed(end),
            open_end: false,
            repeats: None,
        }
    }
}

impl Default for TimeSpan {
    fn default() -> Self {
        Self::fixed_range(
            ExtendedTime::new(0, 0).unwrap(),
            ExtendedTime::new(24, 0).unwrap(),
        )
    }
}

impl Ord for TimeSpan {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.range.start)
            .cmp(&other.range.start)
            .then_with(|| self.range.end.cmp(&other.range.end))
            .then_with(|| self.open_end.cmp(&other.open_end))
            .then_with(|| self.repeats.cmp(&other.repeats))
    }
}

impl PartialOrd for TimeSpan {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for TimeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.range.start)?;

        if !self.open_end || self.range.end != Time::Fixed(ExtendedTime::new(24, 0).unwrap()) {
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

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
