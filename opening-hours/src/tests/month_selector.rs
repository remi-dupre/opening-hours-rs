use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::{datetime, schedule_at, OpeningHours};

#[test]
fn exact_date() -> Result<(), Error> {
    assert_eq!(
        schedule_at!(r#"2020Jun01 open"#, "2020-05-31"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(r#"2020Jun01:10:00-12:10"#, "2020-06-01"),
        schedule! { 10,00 => Open => 12,10 }
    );

    assert_eq!(
        schedule_at!(r#"2020Jun01 open"#, "2020-06-02"),
        schedule! {}
    );

    Ok(())
}

#[test]
fn range() -> Result<(), Error> {
    assert_eq!(
        schedule_at!(r#"Jan-Jun:11:58-11:59"#, "2020-06-01"),
        schedule! { 11,58 => Open => 11,59 }
    );

    assert_eq!(
        schedule_at!(r#"May15-01:10:00-12:00"#, "2020-06-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#"May15-01:10:00-12:00"#, "2020-06-02"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(r#"2019Sep01-2020Jul31:10:00-12:00"#, "2020-06-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#"2019Sep01+:10:00-12:00"#, "2020-06-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#"2019Sep01-Jul01:10:00-12:00"#, "2020-06-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#"Sep01-Jul01:10:00-12:00"#, "2020-06-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    Ok(())
}

#[test]
fn invalid_day() -> Result<(), Error> {
    let oh_oob_february = "Feb01-Feb31:10:00-12:00";
    assert_eq!(schedule_at!(oh_oob_february, "2021-01-31"), schedule! {});

    assert_eq!(
        schedule_at!(oh_oob_february, "2021-02-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(oh_oob_february, "2021-02-28"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(oh_oob_february, "2020-02-29"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(schedule_at!(oh_oob_february, "2021-03-01"), schedule! {});
    Ok(())
}

#[test]
fn jump_month_interval() -> Result<(), Error> {
    let oh = OpeningHours::parse("Jun")?;

    assert_eq!(
        oh.next_change(datetime!("2024-02-15 10:00")).unwrap(),
        datetime!("2024-06-01 00:00")
    );

    assert_eq!(
        oh.next_change(datetime!("2024-06-15 10:00")).unwrap(),
        datetime!("2024-07-01 00:00")
    );

    Ok(())
}

#[test]
fn feb29_point() {
    let oh: OpeningHours = "Feb29".parse().unwrap();

    // 2020 is a leap year
    assert!(!oh.is_open(datetime!("2020-02-28 12:00")));
    assert!(oh.is_open(datetime!("2020-02-29 12:00")));
    assert!(!oh.is_open(datetime!("2020-03-01 12:00")));

    // 2021 is *not* a leap year
    assert!(!oh.is_open(datetime!("2021-02-28 12:00")));
    assert!(!oh.is_open(datetime!("2021-03-01 12:00")));
}

#[test]
fn feb29_starts_interval() {
    let oh: OpeningHours = "Feb29-Mar15".parse().unwrap();

    // 2020 is a leap year
    assert!(!oh.is_open(datetime!("2020-02-28 12:00")));
    assert!(oh.is_open(datetime!("2020-02-29 12:00")));
    assert!(oh.is_open(datetime!("2020-03-01 12:00")));
    assert!(!oh.is_open(datetime!("2020-03-16 12:00")));

    // 2021 is *not* a leap year
    assert!(!oh.is_open(datetime!("2021-02-28 12:00")));
    assert!(oh.is_open(datetime!("2021-03-01 12:00")));
    assert!(!oh.is_open(datetime!("2021-03-16 12:00")));
}

#[test]
fn feb29_ends_interval() {
    let oh: OpeningHours = "Feb15-Feb29".parse().unwrap();

    // 2020 is a leap year
    assert!(!oh.is_open(datetime!("2020-02-14 12:00")));
    assert!(oh.is_open(datetime!("2020-02-15 12:00")));
    assert!(oh.is_open(datetime!("2020-02-29 12:00")));
    assert!(!oh.is_open(datetime!("2020-03-01 12:00")));

    // 2021 is *not* a leap year
    assert!(!oh.is_open(datetime!("2021-02-14 12:00")));
    assert!(oh.is_open(datetime!("2021-02-15 12:00")));
    assert!(oh.is_open(datetime!("2021-02-28 12:00")));
    assert!(!oh.is_open(datetime!("2021-03-01 12:00")));
}
