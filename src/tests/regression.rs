use std::time::Duration;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::opening_hours::DATE_LIMIT;
use crate::tests::exec_with_timeout;
use crate::{datetime, schedule_at, OpeningHours};

#[test]
fn s000_idunn_interval_stops_next_day() -> Result<(), Error> {
    use crate::DateTimeRange;
    use chrono::Duration;

    let oh = OpeningHours::parse("Tu-Su 09:30-18:00; Th 09:30-21:45")?;
    let start = datetime!("2018-06-11 00:00");
    let end = start + Duration::days(1);

    assert_eq!(
        oh.iter_range(start, end).unwrap().collect::<Vec<_>>(),
        vec![DateTimeRange {
            range: start..end,
            kind: Closed,
            comments: vec![].into()
        }],
    );

    Ok(())
}

#[test]
fn s001_idunn_override_weekday() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("Tu-Su 09:30-18:00; Th 09:30-21:45", "2018-06-14"),
        schedule! { 9,30 => Open => 21,45 }
    );

    Ok(())
}

#[test]
fn s002_idunn_override_weekday_keep_unmatched() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("Tu-Su 09:30-18:00; Th 09:30-21:45", "2018-06-15"),
        schedule! { 9,30 => Open => 18,00 }
    );

    Ok(())
}

#[test]
fn s003_idunn_space_separator() {
    assert!(OpeningHours::parse("Jan-Feb 10:00-20:00").is_ok());
}

#[test]
fn s004_idunn_until_midnight_as_00() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("Mo-Su 09:00-00:00 open", "2018-06-14"),
        schedule! { 9,00 => Open => 24,00 }
    );

    Ok(())
}

#[test]
fn s005_idunn_days_cycle() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("We-Mo 11:00-19:00", "2018-06-11"),
        schedule! { 11,00 => Open => 19,00 }
    );

    assert_eq!(
        schedule_at!("We-Mo 11:00-19:00", "2018-06-12"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!("We-Mo 11:00-19:00", "2018-06-13"),
        schedule! { 11,00 => Open => 19,00 }
    );

    assert_eq!(
        schedule_at!("We-Mo 11:00-19:00", "2018-06-14"),
        schedule! { 11,00 => Open => 19,00 }
    );

    Ok(())
}

#[test]
fn s006_idunn_month_cycle() -> Result<(), Error> {
    assert_eq!(
        OpeningHours::parse("Oct-Mar 07:30-19:30; Apr-Sep 07:00-21:00")?
            .next_change(datetime!("2019-02-10 11:00"))
            .unwrap(),
        datetime!("2019-02-10 19:30")
    );

    Ok(())
}

#[test]
fn s007_idunn_date_separator() {
    assert!(OpeningHours::parse(
        "Mo,Th,Sa,Su 09:00-18:00; We,Fr 09:00-21:45; Tu off; Jan 1,May 1,Dec 25: off"
    )
    .is_ok());
}

#[test]
fn s008_pj_no_open_before_separator() {
    assert!(OpeningHours::parse("Mo-Su 00:00-01:00, 07:30-24:00 ; PH off").is_ok());
}

#[test]
fn s009_pj_no_open_before_separator() {
    assert!(OpeningHours::parse(
        "Mo-Su 00:00-01:00, 07:30-24:00 ; PH off ; 2021 Mar 28 00:00-01:00"
    )
    .is_ok());
}

#[test]
fn s010_pj_slow_after_24_7() -> Result<(), Error> {
    exec_with_timeout(Duration::from_millis(100), || {
        OpeningHours::parse("24/7 open ; 2021Jan-Feb off")?
            .next_change(datetime!("2021-07-09 19:30"))
            .unwrap();

        Ok::<(), Error>(())
    })?;

    exec_with_timeout(Duration::from_millis(100), || {
        OpeningHours::parse("24/7 open ; 2021 Jan 01-Feb 10 off")?
            .next_change(datetime!("2021-07-09 19:30"))
            .unwrap();

        Ok::<(), Error>(())
    })?;

    Ok(())
}

#[test]
fn s011_fuzz_extreme_year() -> Result<(), Error> {
    let oh = OpeningHours::parse("2000")?;

    let dt = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(-50_000, 1, 1).unwrap(),
        NaiveTime::default(),
    );

    assert!(oh.is_closed(dt));
    assert_eq!(oh.next_change(dt).unwrap(), DATE_LIMIT);
    Ok(())
}

#[test]
fn s012_fuzz_slow_sh() -> Result<(), Error> {
    exec_with_timeout(Duration::from_millis(100), || {
        OpeningHours::parse("SH")?
            .next_change(datetime!("2020-01-01 00:00"))
            .unwrap();

        Ok(())
    })
}

#[test]
fn s013_fuzz_slow_weeknum() -> Result<(), Error> {
    exec_with_timeout(Duration::from_millis(200), || {
        OpeningHours::parse("Novweek09")?
            .next_change(datetime!("2020-01-01 00:00"))
            .unwrap();

        Ok(())
    })
}

#[test]
fn s014_fuzz_feb30_before_leap_year() -> Result<(), Error> {
    OpeningHours::parse("Feb30")?
        .next_change(datetime!("4419-03-01 00:00"))
        .unwrap();

    Ok(())
}

#[test]
fn s015_fuzz_31dec_may_be_week_01() -> Result<(), Error> {
    assert_eq!(
        OpeningHours::parse("week04")?
            .next_change(datetime!("2024-12-31 12:00"))
            .unwrap(),
        datetime!("2025-01-20 00:00")
    );

    Ok(())
}

#[test]
fn s016_fuzz_week01_sh() -> Result<(), Error> {
    let dt = datetime!("2010-01-03 00:55"); // still week 53 of 2009

    assert_eq!(
        OpeningHours::parse("week01")?.next_change(dt).unwrap(),
        datetime!("2011-01-03 00:00"),
    );

    assert_eq!(
        OpeningHours::parse("week01SH")?.next_change(dt).unwrap(),
        DATE_LIMIT
    );

    Ok(())
}

#[test]
fn s017_fuzz_open_range_timeout() -> Result<(), Error> {
    exec_with_timeout(Duration::from_millis(100), || {
        assert_eq!(
            dbg!(OpeningHours::parse("May2+")?)
                .next_change(datetime!("2020-01-01 12:00"))
                .unwrap(),
            datetime!("2020-05-02 00:00")
        );

        assert_eq!(
            dbg!(OpeningHours::parse("May2+")?)
                .next_change(datetime!("2020-05-15 12:00"))
                .unwrap(),
            datetime!("2021-01-01 00:00")
        );

        Ok(())
    })
}
