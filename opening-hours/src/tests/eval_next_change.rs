use std::str::FromStr;

use rstest::rstest;

use crate::tests::utils::parse::{dt, ParsedDateTime};
use crate::{Context, OpeningHours};

#[rstest]
// Outside of global evaluator bounds
#[case::before_bounds("1789-07-14 12:00", "24/7", "1900-01-01 00:00")]
#[case::before_bounds("1789-07-14 12:00", "3000", "3000-01-01 00:00")]
// Time sector
#[case::time_selector("2024-06-21 22:30", "Jun dusk+", "2024-06-22 00:00")]
// Year ranges
#[case::year_range("2021-02-09 21:00", "2000-3000", "3001-01-01 00:00")]
#[case::year_range("2021-02-09 21:00", "2000-3000/42", "2042-01-01 00:00")]
#[case::year_range("2021-02-09 21:00", "2000-3000/21", "2022-01-01 00:00")]
#[case::year_range("2021-02-09 21:00", "2020,8000-9000 10:00-22:00", "8000-01-01 10:00")]
#[case::year_range("2021-02-09 21:00", "2020,8000-9000 10:00-22:00", "8000-01-01 10:00")]
// Week Range
// . week 52 of 7569 is the last week of the year and ends at the 28th
#[case::week("7569-12-28 08:05", "week 52 ; Jun", "7569-12-29 00:00")]
// . week 52 of 7569 is the last week of the year and ends at the 28th
#[case::week("7569-12-28 08:05", "week 1 ; Jun", "7569-12-29 00:00")]
// . week 52 of 2021 is the last week and ends on the 2th of January
#[case::week("2021-12-28 08:05", "week 52 ; Jun", "2022-01-03 00:00")]
// . week 53 of 2020 ends on 3rd of January
#[case::week("2020-12-28 08:05", "week 53 ; Jun", "2021-01-04 00:00")]
// . there is no week 53 from 2021 to 2026
#[case::week("2021-01-15 08:05", "week 53", "2026-12-28 00:00")]
// Month Selector
#[case::month("2024-02-15 10:00", "Jun", "2024-06-01 00:00")]
#[case::month("2024-06-15 10:00", "Jun", "2024-07-01 00:00")]
#[case::month("2021-02-15 12:00", "2021 Mar 28-Apr 16", "2021-03-28 00:00")]
#[case::month("2021-04-01 12:00", "2021 Mar 28-Apr 16", "2021-04-17 00:00")]
#[case::month("2021-02-15 12:00", "Mar 28-2021 Apr 16", "2021-03-28 00:00")]
#[case::month("2021-04-01 12:00", "Mar 28-2021 Apr 16", "2021-04-17 00:00")]
// Month Selector (with weekday)
#[case::month_wday("2020-01-01 12:00", "Jan Mo[1]-Jan Su[-1]", "2020-01-06 00:00")]
#[case::month_wday("2020-01-06 12:00", "Jan Mo[1]-Jan Su[-1]", "2020-01-27 00:00")]
#[case::month_wday("2020-01-27 12:00", "Jan Mo[1]-Jan Su[-1]", "2021-01-04 00:00")]
#[case::month_wday("2020-01-01 12:00", "easter-2025 Jan Su[-2]", "2024-03-31 00:00")]
#[case::month_wday("2024-03-31 12:00", "easter-2025 Jan Su[-2]", "2025-01-20 00:00")]
#[case::month_wday("2020-01-01 12:00", "Jan Mo[1]-15", "2020-01-06 00:00")]
#[case::month_wday("2020-01-06 12:00", "Jan Mo[1]-15", "2020-01-16 00:00")]
#[case::month_wday("2020-01-16 12:00", "Jan Mo[1]-15", "2021-01-04 00:00")]
// Only the comment changes the state
#[case::comment("2024-01-01 12:00", r#""aaa" ; Mar"#, "2024-03-01 00:00")]
#[case::comment("2024-03-15 12:00", r#""aaa" ; Mar"#, "2024-04-01 00:00")]
#[case::comment(
    "2024-01-01 12:00",
    r#"00:00-14:00 "may open earlier", 14:00-24:00"#,
    "2024-01-01 14:00"
)]
#[case::comment(
    "2024-01-01 16:00",
    r#"00:00-14:00 "may open earlier", 14:00-24:00"#,
    "2024-01-02 00:00"
)]
#[case::comment("2024-01-01 12:00", r#"24/7 "aaa" ; Mar "bbb""#, "2024-03-01 00:00")]
#[case::comment("2024-03-15 12:00", r#"24/7 "aaa" ; Mar "bbb""#, "2024-04-01 00:00")]
#[case::comment("2024-01-01 02:00", r#"01:00-03:00 closed "aaa""#, "2024-01-01 03:00")]
#[case::comment("2024-01-01 12:00", r#"01:00-03:00 closed "aaa""#, "2024-01-02 01:00")]
fn next_change(
    #[case] date: ParsedDateTime,
    #[case] expr: OpeningHours,
    #[case] expected: ParsedDateTime,
) {
    let Some(next_change) = expr.next_change(*date) else {
        panic!("no next change for {expr} after {date}");
    };

    assert_eq!(
        next_change, *expected,
        "wrong next change for {expr} at {date}: {next_change} != {expected}",
    )
}

#[rstest]
// Outside of global evaluator bounds
#[case::after_bounds("9999-01-01 12:00", "24/7")]
// Year ranges
#[case::year_range("2019-02-10 11:00", "24/7")]
#[case::year_range("+10000-01-01 00:00", "24/7")]
// Month selector
#[case::month("2021-04-17 12:00", "2021 Mar 28-Apr 16")]
#[case::month("2021-04-17 12:00", "Mar 28-2021 Apr 16")]
#[case::month_wday("2025-01-20 12:00", "easter-2025 Jan Su[-2]")]
#[case::month_wday("2020-01-01 12:00", "2020 Jan Mo[-1]-15")]
#[case::month_wday("2020-01-01 12:00", "2020 Jan Mo[5]-15")]
fn no_next_change(#[case] date: ParsedDateTime, #[case] expr: OpeningHours) {
    if let Some(next_change) = expr.next_change(*date) {
        panic!("shouldn't have a next change for '{expr}' at '{date}', got {next_change}",)
    }
}

#[test]
fn with_approx_bound_interval_size() {
    let ctx = Context::default().approx_bound_interval_size(chrono::TimeDelta::days(366));

    let oh = OpeningHours::from_str("2024-2030Jun open")
        .unwrap()
        .with_context(ctx);

    assert_eq!(
        oh.next_change(dt("2025-05-01 12:00")).unwrap(),
        dt("2025-06-01 00:00"),
    );

    assert!(oh.next_change(dt("2000-05-01 12:00")).is_none());
    assert!(oh.next_change(dt("2030-07-01 12:00")).is_none());
}
