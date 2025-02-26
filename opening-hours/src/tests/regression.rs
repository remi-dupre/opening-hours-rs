use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::tests::stats::TestStats;
use crate::{datetime, schedule_at, OpeningHours};

#[test]
fn s000_idunn_interval_stops_next_day() -> Result<(), Error> {
    use crate::DateTimeRange;
    use chrono::Duration;

    let oh = "Tu-Su 09:30-18:00; Th 09:30-21:45".parse::<OpeningHours>()?;
    let start = datetime!("2018-06-11 00:00");
    let end = start + Duration::days(1);

    assert_eq!(
        oh.iter_range(start, end).collect::<Vec<_>>(),
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
    assert!("Jan-Feb 10:00-20:00".parse::<OpeningHours>().is_ok());
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
        "Oct-Mar 07:30-19:30; Apr-Sep 07:00-21:00"
            .parse::<OpeningHours>()?
            .next_change(datetime!("2019-02-10 11:00"))
            .unwrap(),
        datetime!("2019-02-10 19:30")
    );

    Ok(())
}

#[test]
fn s007_idunn_date_separator() {
    assert!(
        "Mo,Th,Sa,Su 09:00-18:00; We,Fr 09:00-21:45; Tu off; Jan 1,May 1,Dec 25: off"
            .parse::<OpeningHours>()
            .is_ok()
    );
}

#[test]
fn s008_pj_no_open_before_separator() {
    assert!("Mo-Su 00:00-01:00, 07:30-24:00 ; PH off"
        .parse::<OpeningHours>()
        .is_ok());
}

#[test]
fn s009_pj_no_open_before_separator() {
    assert!(
        "Mo-Su 00:00-01:00, 07:30-24:00 ; PH off ; 2021 Mar 28 00:00-01:00"
            .parse::<OpeningHours>()
            .is_ok()
    );
}

#[test]
fn s010_pj_slow_after_24_7() {
    let stats = TestStats::watch(|| {
        assert!(OpeningHours::parse("24/7 open ; 2021Jan-Feb off")
            .unwrap()
            .next_change(datetime!("2021-07-09 19:30"))
            .is_none());
    });

    assert!(stats.count_generated_schedules < 10);

    let stats = TestStats::watch(|| {
        assert!(OpeningHours::parse("24/7 open ; 2021 Jan 01-Feb 10 off")
            .unwrap()
            .next_change(datetime!("2021-07-09 19:30"))
            .is_none());
    });

    assert!(stats.count_generated_schedules < 10);
}

#[test]
fn s011_fuzz_extreme_year() -> Result<(), Error> {
    let oh: OpeningHours = "2000".parse()?;

    let dt = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(-50_000, 1, 1).unwrap(),
        NaiveTime::default(),
    );

    assert!(oh.is_closed(dt));
    assert_eq!(oh.next_change(dt).unwrap(), datetime!("2000-01-01 00:00"));
    Ok(())
}

#[test]
fn s012_fuzz_slow_sh() {
    let stats = TestStats::watch(|| {
        assert!(OpeningHours::parse("SH")
            .unwrap()
            .next_change(datetime!("2020-01-01 00:00"))
            .is_none());
    });

    assert!(stats.count_generated_schedules < 10);
}

#[test]
fn s013_fuzz_slow_weeknum() {
    let stats = TestStats::watch(|| {
        assert!(OpeningHours::parse("Novweek09")
            .unwrap()
            .next_change(datetime!("2020-01-01 00:00"))
            .is_none());
    });

    assert!(stats.count_generated_schedules < 8000 * 4);
}

#[test]
fn s014_fuzz_feb30_before_leap_year() -> Result<(), Error> {
    assert!(OpeningHours::parse("Feb30")?
        .next_change(datetime!("4419-03-01 00:00"))
        .is_none());

    Ok(())
}

#[test]
fn s015_fuzz_31dec_may_be_week_01() -> Result<(), Error> {
    assert_eq!(
        "week04"
            .parse::<OpeningHours>()?
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
        datetime!("2010-01-04 00:00"),
    );

    assert!(OpeningHours::parse("week01SH")?.next_change(dt).is_none());
    Ok(())
}

#[test]
fn s017_fuzz_open_range_timeout() {
    let stats = TestStats::watch(|| {
        assert_eq!(
            OpeningHours::parse("May2+")
                .unwrap()
                .next_change(datetime!("2020-01-01 12:00"))
                .unwrap(),
            datetime!("2020-05-02 00:00")
        );

        assert_eq!(
            OpeningHours::parse("May2+")
                .unwrap()
                .next_change(datetime!("2020-05-15 12:00"))
                .unwrap(),
            datetime!("2021-01-01 00:00")
        );
    });

    assert!(stats.count_generated_schedules < 10);
}

#[cfg(feature = "auto-country")]
#[cfg(feature = "auto-timezone")]
#[test]
fn s018_fuzz_ph_infinite_loop() -> Result<(), Error> {
    use crate::localization::Coordinates;
    use crate::Context;

    let ctx = Context::from_coords(Coordinates::new(0.0, 4.2619).unwrap());
    let tz = *ctx.locale.get_timezone();
    let oh = OpeningHours::parse("PH")?.with_context(ctx);
    oh.next_change(datetime!("2106-02-12 13:54", tz));
    Ok(())
}

#[test]
fn s019_fuzz_stringify_dusk() -> Result<(), Error> {
    let oh: OpeningHours = "dusk-22:00".parse()?;
    assert!(OpeningHours::parse(&oh.to_string()).is_ok());
    Ok(())
}

#[test]
fn s20_year_starts_at_weeknum_53() -> Result<(), Error> {
    // 1st of January 7583 is in week 53 of previous year which could result
    // on jumping on year 7584 with a failing implementation.
    assert_eq!(
        OpeningHours::parse("week 13")?
            .next_change(datetime!("7583-01-01 12:00"))
            .unwrap(),
        datetime!("7583-03-28 00:00"),
    );

    Ok(())
}
