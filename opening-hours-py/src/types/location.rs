use chrono::{NaiveDate, NaiveDateTime};

use opening_hours::{Localize, NoLocation, TzLocation};
use opening_hours_syntax::rules::time::TimeEvent;

use super::datetime::DateTimeMaybeAware;

#[derive(Clone, PartialEq)]
pub(crate) enum PyLocation {
    Naive,
    Aware(TzLocation<chrono_tz::Tz>),
}

impl Localize for PyLocation {
    type DateTime = DateTimeMaybeAware;

    fn naive(&self, dt: Self::DateTime) -> NaiveDateTime {
        match self {
            PyLocation::Naive => NoLocation.naive(dt.as_naive_local()),
            PyLocation::Aware(loc) => match dt {
                DateTimeMaybeAware::Naive(dt) => dt,
                DateTimeMaybeAware::Aware(dt) => loc.naive(dt),
            },
        }
    }

    fn datetime(&self, naive: NaiveDateTime) -> Self::DateTime {
        match self {
            Self::Naive => DateTimeMaybeAware::Naive(NoLocation.datetime(naive)),
            Self::Aware(loc) => DateTimeMaybeAware::Aware(loc.datetime(naive)),
        }
    }

    fn event_time(&self, date: NaiveDate, event: TimeEvent) -> chrono::NaiveTime {
        match self {
            PyLocation::Naive => NoLocation.event_time(date, event),
            PyLocation::Aware(loc) => loc.event_time(date, event),
        }
    }
}
