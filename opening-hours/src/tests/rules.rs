use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::schedule_at;

#[test]
fn empty() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("", "2020-06-01"),
        schedule! { 00,00 => Open => 24,00 }
    );

    Ok(())
}

#[test]
fn always_open() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("24/7", "2020-06-01"),
        schedule! { 00,00 => Open => 24,00 }
    );

    Ok(())
}

#[test]
fn regular_rule() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("Sa,Su 11:00-13:45 open; 10:00-18:00", "2020-06-01"),
        schedule! { 10,00 => Open => 18,00 }
    );

    assert_eq!(
        schedule_at!("Sa,Su 11:00-13:45 open; 10:00-18:00", "2020-05-31"),
        schedule! { 10,00 => Open => 18,00 }
    );

    Ok(())
}

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
fn fallback_rule() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("Jun:10:00-12:00 open || unknown", "2020-06-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!("Jun:10:00-12:00 open || unknown", "2020-05-31"),
        schedule! { 0,00 => Unknown => 24,00 }
    );

    assert_eq!(
        schedule_at!(
            "Jun:10:00-12:00 open || Mo-Fr closed || unknown",
            "2020-06-01"
        ),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(
            "Jun:10:00-12:00 open || Mo-Fr closed || unknown",
            "2020-05-29"
        ),
        schedule! { 0,00 => Closed => 24,00 }
    );

    assert_eq!(
        schedule_at!(
            "Jun:10:00-12:00 open || Mo-Fr closed || unknown",
            "2020-05-30"
        ),
        schedule! { 0,00 => Unknown => 24,00 }
    );

    Ok(())
}

#[test]
fn comments() -> Result<(), Error> {
    assert_eq!(
        schedule_at!(r#"10:00-12:00 open "welcome!""#, "2020-06-01"),
        schedule! { 10,00 => Open, "welcome!" => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#""42", "31", "53", "53", "42", "01""#, "2020-06-01"),
        schedule! { 0,00 => Open, "01", "31", "42", "53" => 24,00 }
    );

    Ok(())
}
