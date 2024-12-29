use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::schedule_at;

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
    let oh_oob_february = "Feb00-Feb31:10:00-12:00";
    assert_eq!(schedule_at!(oh_oob_february, "2021-01-31"), schedule! {},);

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
