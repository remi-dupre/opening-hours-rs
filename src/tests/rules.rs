use crate::parser::Error;
use crate::schedule_at;
use crate::time_domain::RuleKind::*;

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
