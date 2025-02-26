use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::{datetime, schedule_at, OpeningHours};

#[test]
fn basic_timespan() -> Result<(), Error> {
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

#[test]
fn events() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("(dawn-02:30)-(dusk+02:30)", "2020-06-01"),
        schedule! { 3,30 => Open => 22,30 }
    );

    assert_eq!(
        schedule_at!("(dawn+00:30)-(dusk-00:30)", "2020-06-01"),
        schedule! { 6,30 => Open => 19,30 }
    );

    assert_eq!(
        schedule_at!("sunrise-19:45", "2020-06-01"),
        schedule! { 7,00 => Open => 19,45 }
    );

    assert_eq!(
        schedule_at!("08:15-sunset", "2020-06-01"),
        schedule! { 8,15 => Open => 19,00 }
    );

    Ok(())
}

#[cfg(feature = "auto-timezone")]
#[test]
fn events_localized() -> Result<(), Error> {
    assert_eq!(
        schedule_at!(
            "(dawn-02:30)-(dusk+02:30)",
            "2020-06-01",
            coord = (48.87, 2.29),
        ),
        schedule! { 0,00 => Open => 0,56 ; 2,40 => Open => 24,00 }
    );

    assert_eq!(
        schedule_at!(
            "(dawn+00:30)-(dusk-00:30)",
            "2020-06-01",
            coord = (48.87, 2.29),
        ),
        schedule! { 5,40 => Open => 21,57 }
    );

    assert_eq!(
        schedule_at!("sunrise-19:45", "2020-06-01", coord = (48.87, 2.29)),
        schedule! { 5,51 => Open => 19,45 }
    );

    assert_eq!(
        schedule_at!("08:15-sunset", "2020-06-01", coord = (48.87, 2.29)),
        schedule! { 8,15 => Open => 21,46 }
    );

    Ok(())
}

#[test]
fn overlap() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("10:00-12:00,14:00-25:30", "2020-06-01"),
        schedule! {
            00,00 => Open =>  1,30;
            10,00 => Open => 12,00;
            14,00 => Open => 24,00;
        }
    );

    assert_eq!(
        schedule_at!("Mo 14:00-25:30", "2020-06-02"),
        schedule! { 00,00 => Open =>  1,30 }
    );

    Ok(())
}

#[test]
fn wrapping() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("23:00-01:00", "2020-06-01"),
        schedule! {
            00,00 => Open =>  1,00;
            23,00 => Open => 24,00;
        }
    );

    Ok(())
}

#[test]
fn test_dusk_open_ended() {
    let oh = OpeningHours::parse("Jun dusk+").unwrap();

    assert_eq!(
        oh.next_change(datetime!("2024-06-21 22:30")).unwrap(),
        datetime!("2024-06-22 00:00"),
    );
}

#[test]
fn same_bounds() -> Result<(), Error> {
    let raw_oh = "Mo 04:00-04:00";

    assert_eq!(
        schedule_at!(raw_oh, "2025-02-24"), // Monday
        schedule! { 4,00 => Open => 24,00 },
    );

    assert_eq!(
        schedule_at!(raw_oh, "2025-02-25"), // Tuesday
        schedule! { 00,00 => Open => 4,00 },
    );

    assert_eq!(
        schedule_at!(raw_oh, "2025-02-26"), // Wednesday
        schedule! {},
    );

    Ok(())
}
