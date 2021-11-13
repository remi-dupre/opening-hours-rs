use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::{assert_speed, datetime, schedule_at, OpeningHours};

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
fn s010_pj_slow_after_24_7() {
    assert_speed!(
        OpeningHours::parse("24/7 open ; 2021Jan-Feb off")
            .unwrap()
            .next_change(datetime!("2021-07-09 19:30"))
            .unwrap();
        100 ms
    );

    assert_speed!(
        OpeningHours::parse("24/7 open ; 2021 Jan 01-Feb 10 off")
            .unwrap()
            .next_change(datetime!("2021-07-09 19:30"))
            .unwrap();
        100 ms
    );
}
