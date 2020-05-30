use std::time::Duration;

use chrono::NaiveTime;

pub enum DayEvent {
    Noon,
    Sunrise,
    Sunset,
    Dusk,
}

pub enum PlusMinus {
    Plus,
    Minus,
}

pub enum Time {
    Time(NaiveTime),
    Variable(DayEvent, PlusMinus, Duration),
}

pub struct TimeSelector {
    start: Time,
    end: Option<Time>,
    open_end: bool,
    repeat: Option<Duration>,
}
