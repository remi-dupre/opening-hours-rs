//! Test from Github issues.
//! See https://github.com/remi-dupre/opening-hours-rs/issues

use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind;

use crate::{datetime, DateTimeRange, OpeningHours};

/// https://github.com/remi-dupre/opening-hours-rs/issues/23
#[test]
fn gh023_handling_of_spaces() -> Result<(), Error> {
    let oh = OpeningHours::parse("Apr 1 - Nov 3 00:00-24:00")?;
    let start = datetime!("2018-06-11 00:00");
    let expected_end = datetime!("2018-11-04 00:00");
    assert_eq!(oh.next_change(start).unwrap(), expected_end);
    Ok(())
}

/// https://github.com/remi-dupre/opening-hours-rs/issues/24
#[test]
fn gh024_no_date_range_end_in_intervals() -> Result<(), Error> {
    let oh = OpeningHours::parse("2022 Jan 1-2023 Dec 31")?;
    let start = datetime!("2022-01-01 00:00");
    let expected_end = datetime!("2024-01-01 00:00");
    assert_eq!(oh.next_change(start).unwrap(), expected_end);
    Ok(())
}

/// https://github.com/remi-dupre/opening-hours-rs/issues/45
#[test]
fn gh45_infinite_loop() -> Result<(), Error> {
    let oh = OpeningHours::parse("Jan-Dec")?;
    let start = datetime!("2024-01-01 00:00");
    assert!(oh.next_change(start).is_ok());
    Ok(())
}

/// https://github.com/remi-dupre/opening-hours-rs/issues/52
//
//     November 2024
// Su Mo Tu We Th Fr Sa
//                 1  2
//  3  4  5  6  7  8  9
// 10 11 12 13 14 15 16
// 17 18 19 20 21 22 23
// 24 25 26 27 28 29 30
#[test]
fn gh52_no_interval_after_last_midnight() -> Result<(), Error> {
    let oh = OpeningHours::parse("Mo-Su 00:00-06:00, 23:00-00:00")?;

    let mut intervals = oh
        .iter_range(datetime!("2024-11-11 01:00"), datetime!("2024-11-12 01:00"))
        .unwrap();

    assert_eq!(
        intervals.next(),
        Some(DateTimeRange {
            range: datetime!("2024-11-11 01:00")..datetime!("2024-11-11 06:00"),
            kind: RuleKind::Open,
            comments: Default::default()
        })
    );

    assert_eq!(
        intervals.next(),
        Some(DateTimeRange {
            range: datetime!("2024-11-11 06:00")..datetime!("2024-11-11 23:00"),
            kind: RuleKind::Closed,
            comments: Default::default()
        })
    );

    assert_eq!(
        intervals.next(),
        Some(DateTimeRange {
            range: datetime!("2024-11-11 23:00")..datetime!("2024-11-12 01:00"),
            kind: RuleKind::Open,
            comments: Default::default()
        })
    );

    assert_eq!(intervals.next(), None);
    Ok(())
}
