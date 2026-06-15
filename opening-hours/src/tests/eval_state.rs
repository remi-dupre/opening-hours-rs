use opening_hours_syntax::RuleKind;
use rstest::rstest;

use crate::tests::utils::parse::ParsedDateTime;
use crate::{OpeningHours, RuleKind::*};

#[rstest]
// Date bounds
#[case::bounds("1789-07-14 12:00", "24/7", Closed)]
#[case::bounds("+10000-01-01 12:00", "24/7", Closed)]
// Month Selector (feb29)
#[case::month_feb29("2020-02-28 12:00", "Feb29", Closed)]
#[case::month_feb29("2020-02-29 12:00", "Feb29", Open)]
#[case::month_feb29("2020-03-01 12:00", "Feb29", Closed)]
#[case::month_feb29("2021-02-28 12:00", "Feb29", Closed)]
#[case::month_feb29("2021-03-01 12:00", "Feb29", Closed)]
#[case::month_feb29("2020-02-28 12:00", "Feb29-Mar15", Closed)]
#[case::month_feb29("2020-02-29 12:00", "Feb29-Mar15", Open)]
#[case::month_feb29("2020-03-01 12:00", "Feb29-Mar15", Open)]
#[case::month_feb29("2020-03-16 12:00", "Feb29-Mar15", Closed)]
#[case::month_feb29("2021-02-28 12:00", "Feb29-Mar15", Closed)]
#[case::month_feb29("2021-03-01 12:00", "Feb29-Mar15", Open)]
#[case::month_feb29("2021-03-16 12:00", "Feb29-Mar15", Closed)]
#[case::month_feb29("2020-02-14 12:00", "Feb15-Feb29", Closed)]
#[case::month_feb29("2020-02-15 12:00", "Feb15-Feb29", Open)]
#[case::month_feb29("2020-02-29 12:00", "Feb15-Feb29", Open)]
#[case::month_feb29("2020-03-01 12:00", "Feb15-Feb29", Closed)]
#[case::month_feb29("2021-02-14 12:00", "Feb15-Feb29", Closed)]
#[case::month_feb29("2021-02-15 12:00", "Feb15-Feb29", Open)]
#[case::month_feb29("2021-02-28 12:00", "Feb15-Feb29", Open)]
#[case::month_feb29("2021-03-01 12:00", "Feb15-Feb29", Closed)]
// Easter
#[case::easter("2024-03-30 12:00", "24/7 open ; easter off", Open)]
#[case::easter("2024-03-31 12:00", "24/7 open ; easter off", Closed)]
#[case::easter("2024-04-01 12:00", "24/7 open ; easter off", Open)]
#[case::easter("2023-12-31 12:00", "Jan01-easter", Closed)]
#[case::easter("2024-01-01 12:00", "Jan01-easter", Open)]
#[case::easter("2024-03-30 12:00", "Jan01-easter", Open)]
#[case::easter("2024-03-31 12:00", "Jan01-easter", Open)]
#[case::easter("2024-04-01 12:00", "Jan01-easter", Closed)]
#[case::easter("2024-03-30 12:00", "easter-Dec31", Closed)]
#[case::easter("2024-03-31 12:00", "easter-Dec31", Open)]
#[case::easter("2024-12-31 12:00", "easter-Dec31", Open)]
#[case::easter("2025-01-01 12:00", "easter-Dec31", Closed)]
// Rule: additional
#[case::rule_addional("2023-12-23 12:00", "Su closed || open", Open)]
fn state(
    #[case] date: ParsedDateTime,
    #[case] expr: OpeningHours,
    #[case] expected_state: RuleKind,
) {
    let (state, _comment) = expr.state(*date);

    assert_eq!(
        state, expected_state,
        "wrong state for {expr} at {date}: {state} != {expected_state}",
    )
}

#[rstest]
#[case("open", true)]
#[case("closed", false)]
#[case("unknown", false)]
fn is_open(
    #[values("2020-01-01 12:00")] date: ParsedDateTime,
    #[case] expr: OpeningHours,
    #[case] expected: bool,
) {
    assert_eq!(expr.is_open(*date), expected)
}

#[rstest]
#[case("open", false)]
#[case("closed", true)]
#[case("unknown", false)]
fn is_closed(
    #[values("2020-01-01 12:00")] date: ParsedDateTime,
    #[case] expr: OpeningHours,
    #[case] expected: bool,
) {
    assert_eq!(expr.is_closed(*date), expected)
}

#[rstest]
#[case("open", false)]
#[case("closed", false)]
#[case("unknown", true)]
fn is_unknown(
    #[values("2020-01-01 12:00")] date: ParsedDateTime,
    #[case] expr: OpeningHours,
    #[case] expected: bool,
) {
    assert_eq!(expr.is_unknown(*date), expected)
}
