use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::Display;
use core::ops::Range;

use chrono::Duration;

use crate::display::write_selector;
use crate::extended_time::ExtendedTime;

// TimeSelector

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TimeSelector {
    pub spans: Vec<TimeSpan>,
}

impl TimeSelector {
    /// Note that not all cases can be covered
    pub(crate) fn is_00_24(&self) -> bool {
        self.spans.len() == 1
            && self.spans.first()
                == Some(&TimeSpan::fixed_range(
                    ExtendedTime::MIDNIGHT_00,
                    ExtendedTime::MIDNIGHT_24,
                ))
    }
}

impl TimeSelector {
    #[inline]
    pub fn new(spans: Vec<TimeSpan>) -> Self {
        if spans.is_empty() {
            Self::default()
        } else {
            Self { spans }
        }
    }
}

impl Default for TimeSelector {
    #[inline]
    fn default() -> Self {
        Self {
            spans: vec![TimeSpan::fixed_range(
                ExtendedTime::MIDNIGHT_00,
                ExtendedTime::MIDNIGHT_24,
            )],
        }
    }
}

impl Display for TimeSelector {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write_selector(f, &self.spans)
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

impl Ord for TimeSpan {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.range.start.cmp(&other.range.start))
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.range.start)?;

        if !self.open_end || self.range.end != Time::Fixed(ExtendedTime::MIDNIGHT_24) {
            write!(f, "-{}", self.range.end)?;
        }

        if self.open_end {
            write!(f, "+")?;
        }

        if let Some(repeat) = self.repeats {
            write!(f, "/")?;

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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Fixed(time) => write!(f, "{time}"),
            Self::Variable(time) => write!(f, "{time}"),
        }
    }
}

// VariableTime

// TODO: add unit tests on order
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariableTime {
    pub event: TimeEvent,
    pub offset: i16,
}

impl VariableTime {
    /// Implement a safe partial order for variable times. Some events such as
    /// (dusk+02:00-dawn-02:00) may not be ordered the same way depending on the season.
    pub fn stable_partial_ord(&self, other: &Self) -> Option<Ordering> {
        match (self.event.cmp(&other.event), self.offset.cmp(&other.offset)) {
            (Ordering::Equal, other_cmp) | (other_cmp, Ordering::Equal) => Some(other_cmp),
            (event_cmp, offset_cmp) if event_cmp == offset_cmp => Some(event_cmp),
            _ => None,
        }
    }

    /// Checks is the event is guaranteed to happend before the other one.
    pub fn is_before(&self, other: &Self) -> bool {
        self.stable_partial_ord(&other) == Some(Ordering::Less)
    }
}

impl Display for VariableTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { event, offset } = self;
        let offset_h = offset.abs() / 60;
        let offset_m = offset.abs() % 60;

        match offset.cmp(&0) {
            Ordering::Less => write!(f, "({event}-{offset_h:02}:{offset_m:02})"),
            Ordering::Greater => write!(f, "({event}+{offset_h:02}:{offset_m:02})"),
            Ordering::Equal => write!(f, "{event}"),
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
