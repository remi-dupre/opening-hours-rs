use std::ops::Add;

use chrono::{DateTime, Local, NaiveDateTime, TimeDelta};
use pyo3::prelude::*;

use opening_hours::opening_hours::DATE_LIMIT;
use opening_hours::{Localize, TzLocation};

#[derive(Clone, Copy, FromPyObject, IntoPyObject)]
pub(crate) enum DateTimeMaybeAware {
    Naive(NaiveDateTime),
    Aware(DateTime<chrono_tz::Tz>),
}

impl DateTimeMaybeAware {
    /// Drop eventual timezone information.
    pub(crate) fn as_naive_local(&self) -> NaiveDateTime {
        match self {
            DateTimeMaybeAware::Naive(naive_date_time) => *naive_date_time,
            DateTimeMaybeAware::Aware(date_time) => date_time.naive_local(),
        }
    }

    /// Just ensures that *DATE_LIMIT* is mapped to `None`.
    pub(crate) fn map_date_limit(self) -> Option<Self> {
        if self.as_naive_local() == DATE_LIMIT {
            None
        } else {
            Some(self)
        }
    }

    /// Fetch local time if value is `None`.
    pub(crate) fn unwrap_or_now(val: Option<Self>) -> Self {
        val.unwrap_or_else(|| Self::Naive(Local::now().naive_local()))
    }

    pub(crate) fn timezone(&self) -> Option<chrono_tz::Tz> {
        match self {
            Self::Naive(_) => None,
            Self::Aware(dt) => Some(dt.timezone()),
        }
    }

    pub(crate) fn or_with_timezone(self, tz: chrono_tz::Tz) -> Self {
        match self {
            Self::Naive(dt) => Self::Aware(TzLocation::new(tz).datetime(dt)),
            Self::Aware(_) => self,
        }
    }

    pub(crate) fn or_with_timezone_of(self, other: Self) -> Self {
        match other {
            Self::Naive(_) => self,
            Self::Aware(dt) => self.or_with_timezone(dt.timezone()),
        }
    }
}

impl Add<TimeDelta> for DateTimeMaybeAware {
    type Output = Self;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        match self {
            DateTimeMaybeAware::Naive(dt) => DateTimeMaybeAware::Naive(dt + rhs),
            DateTimeMaybeAware::Aware(dt) => DateTimeMaybeAware::Aware(dt + rhs),
        }
    }
}
