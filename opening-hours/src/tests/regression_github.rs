//! Tests that proove fixes of past Github issues.
//! See https://github.com/remi-dupre/opening-hours-rs/issues

use std::str::FromStr;

use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind;

use crate::{tests::utils::parse::dt, DateTimeRange, OpeningHours};

/// https://github.com/remi-dupre/opening-hours-rs/issues/23
#[test]
fn gh023_handling_of_spaces() -> Result<(), Error> {
    let oh: OpeningHours = "Apr 1 - Nov 3 00:00-24:00".parse()?;
    let start = dt("2018-06-11 00:00");
    let expected_end = dt("2018-11-04 00:00");
    assert_eq!(oh.next_change(start).unwrap(), expected_end);
    Ok(())
}

/// https://github.com/remi-dupre/opening-hours-rs/issues/24
#[test]
fn gh024_no_date_range_end_in_intervals() -> Result<(), Error> {
    let oh: OpeningHours = "2022 Jan 1-2023 Dec 31".parse()?;
    let start = dt("2022-01-01 00:00");
    let expected_end = dt("2024-01-01 00:00");
    assert_eq!(oh.next_change(start).unwrap(), expected_end);
    Ok(())
}

/// https://github.com/remi-dupre/opening-hours-rs/issues/45
#[test]
fn gh45_infinite_loop() -> Result<(), Error> {
    let oh: OpeningHours = "Jan-Dec".parse()?;
    let start = dt("2024-01-01 00:00");
    assert!(oh.next_change(start).is_none());
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
    let oh = OpeningHours::from_str("Mo-Su 00:00-06:00, 23:00-00:00")?;
    let mut intervals = oh.iter_range(dt("2024-11-11 01:00"), dt("2024-11-12 01:00"));

    assert_eq!(
        intervals.next(),
        Some(DateTimeRange {
            range: dt("2024-11-11 01:00")..dt("2024-11-11 06:00"),
            kind: RuleKind::Open,
            comment: Default::default()
        })
    );

    assert_eq!(
        intervals.next(),
        Some(DateTimeRange {
            range: dt("2024-11-11 06:00")..dt("2024-11-11 23:00"),
            kind: RuleKind::Closed,
            comment: Default::default()
        })
    );

    assert_eq!(
        intervals.next(),
        Some(DateTimeRange {
            range: dt("2024-11-11 23:00")..dt("2024-11-12 01:00"),
            kind: RuleKind::Open,
            comment: Default::default()
        })
    );

    assert_eq!(intervals.next(), None);
    Ok(())
}

/// https://github.com/remi-dupre/opening-hours-rs/issues/56
#[test]
fn gh56_only_close_when_no_day_filter() -> Result<(), Error> {
    let oh: OpeningHours = "00:30-05:30".parse()?;
    let date_start = dt("2024-11-25 17:30");
    let date_end = dt("2024-11-26 09:00");
    let mut intervals = oh.iter_range(date_start, date_end);

    assert_eq!(
        intervals.next().unwrap(),
        DateTimeRange {
            range: date_start..dt("2024-11-26 00:30"),
            kind: RuleKind::Closed,
            comment: Default::default(),
        }
    );

    Ok(())
}

/// https://github.com/remi-dupre/opening-hours-rs/issues/77
#[test]
fn gh77_invalid_time_step_panics() {
    OpeningHours::from_str("Mo-Sa 09:00-20:00/21:00").unwrap();
}

/// https://github.com/remi-dupre/opening-hours-rs/issues/88
///
/// A comma separator in a time block should have precedence over a additional
/// rule separator, even when there is a space (source: JS library).
#[test]
fn gh88_allow_space_in_time_block() {
    let oh_1 =
        OpeningHours::from_str("Mo 11:45-14:30; Tu-Fr 11:45-14:30, 19:00-21:45; Sa 19:00-21:45")
            .unwrap();

    let oh_2 =
        OpeningHours::from_str("Mo 11:45-14:30; Tu-Fr 11:45-14:30,19:00-21:45; Sa 19:00-21:45")
            .unwrap();

    assert_eq!(oh_1.normalize(), oh_2.normalize());
}

/// https://github.com/remi-dupre/opening-hours-rs/issues/97
#[test]
fn gh97_normalize_double_overlap() {
    let oh =
        OpeningHours::from_str("Mo 13:00-15:00; Tu 08:00-10:00; We 08:00-10:00, We 13:00-15:00")
            .unwrap();

    let normalized = oh.normalize();

    // From monday to sunday
    let dates: Vec<_> = (11..=17)
        .map(|day| format!("2026-05-{day}").parse().unwrap())
        .collect();

    for date in dates {
        assert_eq!(oh.schedule_at(date), normalized.schedule_at(date));
    }
}
