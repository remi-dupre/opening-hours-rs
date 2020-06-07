use crate::parser::Error;
use crate::schedule_at;
use crate::time_domain::RuleKind::*;

#[test]
fn monthday_range_exact_date() -> Result<(), Error> {
    assert_eq!(
        schedule_at!(r#"2020Jun01 open"#, "2020-05-31"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(r#"2020Jun01"#, "2020-06-01"),
        schedule! { 00,00 => Open => 24,00 }
    );

    assert_eq!(
        schedule_at!(r#"2020Jun01 open"#, "2020-06-02"),
        schedule! {}
    );

    Ok(())
}
