use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::schedule_at;

#[test]
fn basic_range() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("Mo-Su", "2020-06-01"),
        schedule! { 00,00 => Open => 24,00 }
    );

    assert_eq!(schedule_at!("Tu", "2020-06-01"), schedule! {});

    assert_eq!(
        schedule_at!("Tu", "2020-06-02"),
        schedule! { 00,00 => Open => 24,00 }
    );

    assert_eq!(schedule_at!("We", "2020-06-02"), schedule! {});

    for date in &[
        "2020-06-01",
        "2020-06-02",
        "2020-06-04",
        "2020-06-06",
        "2020-06-07",
    ] {
        assert_eq!(
            schedule_at!("Mo-Tu,Th,Sa-Su 10:00-12:00", date),
            schedule! { 10,00 => Open => 12,00 }
        );
    }

    for date in &["2020-06-03", "2020-06-05"] {
        assert_eq!(
            schedule_at!("Mo-Tu,Th,Sa-Su 10:00-12:00", date),
            schedule! {}
        );
    }

    Ok(())
}

#[test]
fn nth() -> Result<(), Error> {
    for date in &["2020-06-08", "2020-06-15", "2020-06-22"] {
        assert_eq!(
            schedule_at!("Mo[2-4] 10:00-12:00", date),
            schedule! { 10,00 => Open => 12,00 }
        );
    }

    for date in &["2020-06-01", "2020-06-29"] {
        assert_eq!(schedule_at!("Mo[2-4] 10:00-12:00", date), schedule! {});
    }

    assert_eq!(
        schedule_at!("Mo[1] 10:00-12:00", "2020-06-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!("Mo[1] 10:00-12:00", "2020-06-08"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!("Mo[1] 10:00-12:00", "2020-06-02"),
        schedule! {}
    );

    Ok(())
}

#[test]
fn nth_with_offset() -> Result<(), Error> {
    for date in &["2020-06-10", "2020-06-17", "2020-06-24"] {
        assert_eq!(
            schedule_at!("Mo[2-4] +2 days 10:00-12:00", date),
            schedule! { 10,00 => Open => 12,00 }
        );
    }

    for date in &["2020-06-03", "2020-07-01"] {
        assert_eq!(
            schedule_at!("Mo[2-4] +2 days 10:00-12:00", date),
            schedule! {}
        );
    }

    assert_eq!(
        schedule_at!("Mo[1] -1 day 10:00-12:00", "2020-05-31"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!("Mo[1] -1 day 10:00-12:00", "2020-06-01"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!("Mo[1] -1 day 10:00-12:00", "2020-06-07"),
        schedule! {}
    );

    Ok(())
}

fn _holiday() {
    // TODO
}
