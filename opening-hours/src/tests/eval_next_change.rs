use chrono::NaiveDateTime;
use rstest::rstest;

use crate::schedule::Schedule;
use crate::tests::utils::parse::dt;
use crate::tests::utils::stats::TestStats;
use crate::{Context, OpeningHours};

#[rstest]
// Outside of global evaluator bounds
#[case::before_bounds(dt("1789-07-14 12:00"), "24/7", dt("1900-01-01 00:00"))]
#[case::before_bounds(dt("1789-07-14 12:00"), "3000", dt("3000-01-01 00:00"))]
// Time sector
#[case::time_selector(dt("2024-06-21 22:30"), "Jun dusk+", dt("2024-06-22 00:00"))]
// Year ranges
#[case::year_range(dt("2021-02-09 21:00"), "2000-3000", dt("3001-01-01 00:00"))]
#[case::year_range(dt("2021-02-09 21:00"), "2000-3000/42", dt("2042-01-01 00:00"))]
#[case::year_range(dt("2021-02-09 21:00"), "2000-3000/21", dt("2022-01-01 00:00"))]
#[case::year_range(
    dt("2021-02-09 21:00"),
    "2020,8000-9000 10:00-22:00",
    dt("8000-01-01 10:00")
)]
#[case::year_range(
    dt("2021-02-09 21:00"),
    "2020,8000-9000 10:00-22:00",
    dt("8000-01-01 10:00")
)]
// Week Range
// . week 52 of 7569 is the last week of the year and ends at the 28th
#[case::week(dt("7569-12-28 08:05"), "week 52 ; Jun", dt("7569-12-29 00:00"))]
// . week 52 of 7569 is the last week of the year and ends at the 28th
#[case::week(dt("7569-12-28 08:05"), "week 1 ; Jun", dt("7569-12-29 00:00"))]
// . week 52 of 2021 is the last week and ends on the 2th of January
#[case::week(dt("2021-12-28 08:05"), "week 52 ; Jun", dt("2022-01-03 00:00"))]
// . week 53 of 2020 ends on 3rd of January
#[case::week(dt("2020-12-28 08:05"), "week 53 ; Jun", dt("2021-01-04 00:00"))]
// . there is no week 53 from 2021 to 2026
#[case::week(dt("2021-01-15 08:05"), "week 53", dt("2026-12-28 00:00"))]
// Month Range
#[case::month(dt("2024-02-15 10:00"), "Jun", dt("2024-06-01 00:00"))]
#[case::month(dt("2024-06-15 10:00"), "Jun", dt("2024-07-01 00:00"))]
#[case::month(dt("2021-02-15 12:00"), "2021 Mar 28-Apr 16", dt("2021-03-28 00:00"))]
#[case::month(dt("2021-04-01 12:00"), "2021 Mar 28-Apr 16", dt("2021-04-17 00:00"))]
#[case::month(dt("2021-02-15 12:00"), "Mar 28-2021 Apr 16", dt("2021-03-28 00:00"))]
#[case::month(dt("2021-04-01 12:00"), "Mar 28-2021 Apr 16", dt("2021-04-17 00:00"))]
// Only the comment changes the state
#[case::comment(dt("2024-01-01 12:00"), r#""aaa" ; Mar"#, dt("2024-03-01 00:00"))]
#[case::comment(dt("2024-03-15 12:00"), r#""aaa" ; Mar"#, dt("2024-04-01 00:00"))]
#[case::comment(
    dt("2024-01-01 12:00"),
    r#"00:00-14:00 "may open earlier", 14:00-24:00"#,
    dt("2024-01-01 14:00")
)]
#[case::comment(
    dt("2024-01-01 16:00"),
    r#"00:00-14:00 "may open earlier", 14:00-24:00"#,
    dt("2024-01-02 00:00")
)]
#[case::comment(
    dt("2024-01-01 12:00"),
    r#"24/7 "aaa" ; Mar "bbb""#,
    dt("2024-03-01 00:00")
)]
#[case::comment(
    dt("2024-03-15 12:00"),
    r#"24/7 "aaa" ; Mar "bbb""#,
    dt("2024-04-01 00:00")
)]
#[case::comment(
    dt("2024-01-01 02:00"),
    r#"01:00-03:00 closed "aaa""#,
    dt("2024-01-01 03:00")
)]
#[case::comment(
    dt("2024-01-01 12:00"),
    r#"01:00-03:00 closed "aaa""#,
    dt("2024-01-02 01:00")
)]
fn next_change(
    #[case] date: NaiveDateTime,
    #[case] expr: OpeningHours,
    #[case] expected: NaiveDateTime,
) {
    let next_change = expr.next_change(date).expect("should have a next change");

    assert_eq!(
        next_change, expected,
        "wrong next change for {expr} at {date}: {next_change} != {expected}",
    )
}

#[rstest]
// Outside of global evaluator bounds
#[case::after_bounds(dt("9999-01-01 12:00"), "24/7")]
// Year ranges
#[case::year_range(dt("2019-02-10 11:00"), "24/7")]
#[case::year_range(dt("+10000-01-01 00:00"), "24/7")]
// Month selector
#[case::month(dt("2021-04-17 12:00"), "2021 Mar 28-Apr 16")]
#[case::month(dt("2021-04-17 12:00"), "Mar 28-2021 Apr 16")]
fn no_next_change(#[case] date: NaiveDateTime, #[case] expr: OpeningHours) {
    assert!(
        expr.next_change(date).is_none(),
        "shouldn't have a next change for '{expr}' at '{date}'",
    )
}

#[test]
fn with_approx_bound_interval_size() {
    let ctx = Context::default().approx_bound_interval_size(chrono::TimeDelta::days(366));

    let oh = OpeningHours::parse("2024-2030Jun open")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(dt("2025-05-01 12:00")).unwrap(),
        dt("2025-06-01 00:00"),
    );

    assert!(oh.next_change(dt("2000-05-01 12:00")).is_none());
    assert!(oh.next_change(dt("2030-07-01 12:00")).is_none());
}

#[test]
fn explicit_closed_slow() {
    let stats = TestStats::watch(|| {
        assert!(OpeningHours::parse("Feb Fr off")
            .unwrap()
            .next_change(dt("2021-07-09 19:30"))
            .is_none());
    });

    assert!(stats.count_generated_schedules < 10);
}
