use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::schedule_at;

#[test]
fn week_range() -> Result<(), Error> {
    assert_eq!(
        schedule_at!(r#"week01:10:00-12:00"#, "2020-01-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#"week01:10:00-12:00"#, "2020-01-06"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(r#"week01,23-24:10:00-12:00"#, "2020-01-06"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(r#"week01,22-23:10:00-12:00"#, "2020-05-31"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#"week01,22-23:10:00-12:00"#, "2020-06-07"),
        schedule! { 10,00 => Open => 12,00 }
    );

    for date in &["2020-01-01", "2020-01-15", "2020-01-29"] {
        assert_eq!(
            schedule_at!(r#"week01-53/2:10:00-12:00"#, date),
            schedule! { 10,00 => Open => 12,00 }
        );
    }

    for date in &["2020-01-08", "2020-01-22"] {
        assert_eq!(
            schedule_at!(r#"week01-53/2:10:00-12:00"#, date),
            schedule! {}
        );
    }

    Ok(())
}
