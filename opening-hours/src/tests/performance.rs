use chrono::TimeDelta;
use rstest::rstest;

use crate::tests::utils::parse::ParsedDateTime;
use crate::tests::utils::stats::TestStats;
use crate::{Context, OpeningHours};

#[rstest]
#[case(10, "2021-07-09 19:30", "Feb Fr off")]
fn next_change(
    #[case] max_generated_schedules: u64,
    #[case] dt: ParsedDateTime,
    #[case] expr: OpeningHours,
) {
    let stats = TestStats::watch(|| {
        assert!(expr.next_change(*dt).is_none());
    });

    assert!(
        stats.count_generated_schedules < max_generated_schedules,
        "Next change on '{}' generated {} schedules",
        expr,
        stats.count_generated_schedules
    );
}

#[rstest]
#[case(400, "2020-06-01 12:00", "2026 Jan Mo[1]-2026 Jan Fr[1] Sa-Su")]
fn next_change_approx(
    #[case] max_generated_schedules: u64,
    #[case] dt: ParsedDateTime,
    #[case] expr: OpeningHours,
) {
    let expr =
        expr.with_context(Context::default().approx_bound_interval_size(TimeDelta::days(366)));

    let stats = TestStats::watch(|| {
        assert!(expr.next_change(*dt).is_none());
    });

    assert!(
        stats.count_generated_schedules < max_generated_schedules,
        "Next change on '{}' generated {} schedules",
        expr,
        stats.count_generated_schedules
    );
}
