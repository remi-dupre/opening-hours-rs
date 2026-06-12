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

/// Shortcut to build a datetime
pub(crate) fn dt(expr: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(expr, "%Y-%m-%d %H:%M").expect("invalid datetime literal")
}

/// Shortcut to build a datetime with a timezone
pub(crate) fn dtz<Tz: TimeZone>(expr: &str, tz: Tz) -> DateTime<Tz> {
    NaiveDateTime::parse_from_str(expr, "%Y-%m-%d %H:%M")
        .expect("invalid datetime literal")
        .and_local_timezone(tz)
        .single()
        .expect("ambiguous datetime on timezone")
}
