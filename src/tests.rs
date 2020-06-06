use chrono::NaiveDate;

use crate::parser::{parse, Error};
use crate::time_domain::RuleKind::*;

fn test_date_1() -> NaiveDate {
    NaiveDate::from_ymd(2020, 6, 1)
}

#[test]
fn empty() -> Result<(), Error> {
    assert_eq!(parse("")?.schedule_at(test_date_1()), schedule! {});
    Ok(())
}

#[test]
fn time_selector_only() -> Result<(), Error> {
    assert_eq!(
        parse("14:00-19:00")?.schedule_at(test_date_1()),
        schedule! { 14,00 => Open => 19,00 }
    );

    assert_eq!(
        parse("10:00-12:00,14:00-16:00")?.schedule_at(test_date_1()),
        schedule! {
            10,00 => Open => 12,00;
            14,00 => Open => 16,00;
        }
    );

    assert_eq!(
        parse("10:00-12:00,11:00-16:00 unknown")?.schedule_at(test_date_1()),
        schedule! { 10,00 => Unknown => 16,00 }
    );

    Ok(())
}

#[test]
fn additional_rule() -> Result<(), Error> {
    assert_eq!(
        parse("10:00-12:00 open, 14:00-16:00 unknown, 16:00-23:00 closed")?
            .schedule_at(test_date_1()),
        schedule! {
            10,00 => Open => 12,00;
            14,00 => Unknown => 16,00;
            16,00 => Closed => 23,00;
        }
    );

    assert_eq!(
        parse("10:00-20:00 open, 12:00-14:00 closed")?.schedule_at(test_date_1()),
        schedule! { 10,00 => Open => 12,00 => Closed => 14,00 => Open => 20,00 }
    );

    assert_eq!(
        parse("12:00-14:00 closed, 10:00-20:00 open")?.schedule_at(test_date_1()),
        schedule! { 10,00 => Open => 20,00 }
    );

    Ok(())
}

#[test]
fn comments() -> Result<(), Error> {
    assert_eq!(
        parse(r#"10:00-12:00 open "welcome!""#)?.schedule_at(test_date_1()),
        schedule! { 10,00 => Open, "welcome!" => 12,00 }
    );

    Ok(())
}
