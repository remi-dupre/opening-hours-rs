//! Tests that proove fixes of past integration issues.

use std::str::FromStr;

use chrono::Duration;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::schedule::Schedule;
use crate::tests::utils::parse::dt;
use crate::tests::utils::stats::TestStats;
use crate::DateTimeRange;
use crate::OpeningHours;

#[test]
fn s000_idunn_interval_stops_next_day() {
    let oh = OpeningHours::from_str("Tu-Su 09:30-18:00; Th 09:30-21:45").unwrap();
    let start = dt("2018-06-11 00:00");
    let end = start + Duration::days(1);

    assert_eq!(
        oh.iter_range(start, end).collect::<Vec<_>>(),
        vec![DateTimeRange {
            range: start..end,
            kind: Closed,
            comment: Default::default(),
        }],
    );
}

#[test]
fn s001_idunn_override_weekday() {
    let oh = OpeningHours::from_str("Tu-Su 09:30-18:00; Th 09:30-21:45").unwrap();

    assert_eq!(
        oh.schedule_at("2018-06-14".parse().unwrap()),
        "09:30 open 21:45".parse().unwrap()
    );
}

#[test]
fn s002_idunn_override_weekday_keep_unmatched() {
    let oh = OpeningHours::from_str("Tu-Su 09:30-18:00; Th 09:30-21:45").unwrap();

    assert_eq!(
        oh.schedule_at("2018-06-15".parse().unwrap()),
        "09:30 open 18:00".parse().unwrap()
    );
}

#[test]
fn s003_idunn_space_separator() {
    assert!(OpeningHours::from_str("Jan-Feb 10:00-20:00").is_ok());
}

#[test]
fn s004_idunn_until_midnight_as_00() {
    let oh = OpeningHours::from_str("Mo-Su 09:00-00:00 open").unwrap();

    assert_eq!(
        oh.schedule_at("2018-06-14".parse().unwrap()),
        "09:00 open 24:00".parse().unwrap()
    );
}

#[test]
fn s005_idunn_days_cycle() {
    let oh = OpeningHours::from_str("We-Mo 11:00-19:00").unwrap();

    assert_eq!(
        oh.schedule_at("2018-06-11".parse().unwrap()),
        "11:00 open 19:00".parse().unwrap()
    );

    assert_eq!(
        oh.schedule_at("2018-06-12".parse().unwrap()),
        Schedule::new()
    );

    assert_eq!(
        oh.schedule_at("2018-06-13".parse().unwrap()),
        "11:00 open 19:00".parse().unwrap()
    );

    assert_eq!(
        oh.schedule_at("2018-06-14".parse().unwrap()),
        "11:00 open 19:00".parse().unwrap()
    );
}

#[test]
fn s006_idunn_month_cycle() {
    assert_eq!(
        OpeningHours::from_str("Oct-Mar 07:30-19:30; Apr-Sep 07:00-21:00")
            .unwrap()
            .next_change(dt("2019-02-10 11:00"))
            .unwrap(),
        dt("2019-02-10 19:30")
    );
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
            .next_change(dt("2021-07-09 19:30"))
            .is_none());
    });

    assert!(stats.count_generated_schedules < 10);

    let stats = TestStats::watch(|| {
        assert!(OpeningHours::parse("24/7 open ; 2021 Jan 01-Feb 10 off")
            .unwrap()
            .next_change(dt("2021-07-09 19:30"))
            .is_none());
    });

    assert!(stats.count_generated_schedules < 10);
}

#[test]
fn s011_fuzz_extreme_year() {
    let oh = OpeningHours::from_str("2000").unwrap();

    let now = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(-50_000, 1, 1).unwrap(),
        NaiveTime::default(),
    );

    assert!(oh.is_closed(now));
    assert_eq!(oh.next_change(now).unwrap(), dt("2000-01-01 00:00"));
}

#[test]
fn s012_fuzz_slow_sh() {
    let stats = TestStats::watch(|| {
        assert!(OpeningHours::parse("SH")
            .unwrap()
            .next_change(dt("2020-01-01 00:00"))
            .is_none());
    });

    assert!(stats.count_generated_schedules < 10);
}

#[test]
fn s013_fuzz_slow_weeknum() {
    let stats = TestStats::watch(|| {
        assert!(OpeningHours::parse("Novweek09")
            .unwrap()
            .next_change(dt("2020-01-01 00:00"))
            .is_none());
    });

    assert!(stats.count_generated_schedules < 8000 * 4);
}

#[test]
fn s014_fuzz_feb30_before_leap_year() {
    assert!(OpeningHours::parse("Feb30")
        .unwrap()
        .next_change(dt("4419-03-01 00:00"))
        .is_none());
}

#[test]
fn s015_fuzz_31dec_may_be_week_01() {
    assert_eq!(
        OpeningHours::from_str("week04")
            .unwrap()
            .next_change(dt("2024-12-31 12:00"))
            .unwrap(),
        dt("2025-01-20 00:00")
    );
}

#[test]
fn s016_fuzz_week01_sh() {
    let now = dt("2010-01-03 00:55"); // still week 53 of 2009

    assert_eq!(
        OpeningHours::from_str("week01")
            .unwrap()
            .next_change(now)
            .unwrap(),
        dt("2010-01-04 00:00"),
    );

    assert!(OpeningHours::parse("week01SH")
        .unwrap()
        .next_change(now)
        .is_none());
}

#[test]
fn s017_fuzz_open_range_timeout() {
    let stats = TestStats::watch(|| {
        assert_eq!(
            OpeningHours::parse("May2+")
                .unwrap()
                .next_change(dt("2020-01-01 12:00"))
                .unwrap(),
            dt("2020-05-02 00:00")
        );

        assert_eq!(
            OpeningHours::parse("May2+")
                .unwrap()
                .next_change(dt("2020-05-15 12:00"))
                .unwrap(),
            dt("2021-01-01 00:00")
        );
    });

    assert!(stats.count_generated_schedules < 10);
}

#[cfg(feature = "auto-country")]
#[cfg(feature = "auto-timezone")]
#[test]
fn s018_fuzz_ph_infinite_loop() {
    use crate::localization::Coordinates;
    use crate::tests::utils::parse::dtz;
    use crate::Context;

    let ctx = Context::from_coords(Coordinates::new(0.0, 4.2619).unwrap());
    let tz = *ctx.locale.get_timezone();
    let oh = OpeningHours::parse("PH").unwrap().with_context(ctx);
    oh.next_change(dtz("2106-02-12 13:54", tz));
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
            .next_change(dt("7583-01-01 12:00"))
            .unwrap(),
        dt("7583-03-28 00:00"),
    );

    Ok(())
}
