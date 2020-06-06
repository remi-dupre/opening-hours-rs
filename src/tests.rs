use chrono::NaiveDate;

use crate::parser::{parse, Error};
use crate::time_domain::RuleKind::*;

macro_rules! schedule_at {
    ( $expression:expr, $date:expr ) => {
        parse($expression)?
            .schedule_at(NaiveDate::parse_from_str($date, "%Y-%m-%d").expect("invalid date"))
    };
}

#[test]
fn empty() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("", "2020-06-01"),
        schedule! { 00,00 => Open => 24,00 }
    );

    Ok(())
}

// Rules

#[test]
fn additional_rule() -> Result<(), Error> {
    assert_eq!(
        schedule_at!(
            "10:00-12:00 open, 14:00-16:00 unknown, 16:00-23:00 closed",
            "2020-06-01"
        ),
        schedule! {
            10,00 => Open => 12,00;
            14,00 => Unknown => 16,00;
            16,00 => Closed => 23,00;
        }
    );

    assert_eq!(
        schedule_at!("10:00-20:00 open, 12:00-14:00 closed", "2020-06-01"),
        schedule! { 10,00 => Open => 12,00 => Closed => 14,00 => Open => 20,00 }
    );

    assert_eq!(
        schedule_at!("12:00-14:00 closed, 10:00-20:00 open", "2020-06-01"),
        schedule! { 10,00 => Open => 20,00 }
    );

    Ok(())
}

#[test]
fn comments() -> Result<(), Error> {
    assert_eq!(
        schedule_at!(r#"10:00-12:00 open "welcome!""#, "2020-06-01"),
        schedule! { 10,00 => Open, "welcome!" => 12,00 }
    );

    Ok(())
}

// Time selector

#[test]
fn time_selector() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("14:00-19:00", "2020-06-01"),
        schedule! { 14,00 => Open => 19,00 }
    );

    assert_eq!(
        schedule_at!("10:00-12:00,14:00-16:00", "2020-06-01"),
        schedule! {
            10,00 => Open => 12,00;
            14,00 => Open => 16,00;
        }
    );

    assert_eq!(
        schedule_at!("10:00-12:00,11:00-16:00 unknown", "2020-06-01"),
        schedule! { 10,00 => Unknown => 16,00 }
    );

    Ok(())
}

// Month Selector

#[test]
fn monthday_range_exact_date() -> Result<(), Error> {
    assert_eq!(
        schedule_at!(r#"2020Jun01 open"#, "2020-05-31"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(r#"2020Jun01"#, "2020-06-01"),
        schedule! { 00,00 => Open => 24,00 }
    );

    assert_eq!(
        schedule_at!(r#"2020Jun01 open"#, "2020-06-02"),
        schedule! {}
    );

    Ok(())
}
