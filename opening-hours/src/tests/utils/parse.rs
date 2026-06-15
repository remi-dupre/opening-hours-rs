use std::{fmt::Display, ops::Deref, str::FromStr};

use chrono::{DateTime, NaiveDateTime, TimeZone};
use opening_hours_syntax::ExtendedTime;

/// Shortcut to build extended time
pub(crate) fn xt(expr: &str) -> ExtendedTime {
    let (hours, minutes) = expr.split_once(":").expect("missing time separator");

    ExtendedTime::new(
        hours.parse().expect("invalid hours"),
        minutes.parse().expect("invalid minutes"),
    )
    .expect("invalid extended time")
}

/// NaiveDateTime wrapper with a simpler parse syntax
pub(crate) struct ParsedDateTime(NaiveDateTime);

impl FromStr for ParsedDateTime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")?))
    }
}

impl Deref for ParsedDateTime {
    type Target = NaiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ParsedDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Shortcut to build a datetime
pub(crate) fn dt(expr: &str) -> NaiveDateTime {
    *ParsedDateTime::from_str(expr).expect("invalid datetime literal")
}

/// Shortcut to build a datetime with a timezone
pub(crate) fn dtz<Tz: TimeZone>(expr: &str, tz: Tz) -> DateTime<Tz> {
    NaiveDateTime::parse_from_str(expr, "%Y-%m-%d %H:%M")
        .expect("invalid datetime literal")
        .and_local_timezone(tz)
        .single()
        .expect("ambiguous datetime on timezone")
}
