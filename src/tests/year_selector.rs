use crate::schedule_at;
use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

#[test]
fn range() -> Result<(), Error> {
    assert_eq!(
        schedule_at!(r#"2020:10:00-12:00"#, "2020-01-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#"2020:10:00-12:00"#, "2021-01-01"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(r#"2010-2019,2021,2025+:10:00-12:00"#, "2020-01-01"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(r#"2010-2019,2021,2025+:10:00-12:00"#, "2024-01-01"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(r#"2010-2019,2021,2025+:10:00-12:00"#, "2015-01-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#"2010-2019,2021,2025+:10:00-12:00"#, "5742-01-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#"2010-2100/3:10:00-12:00"#, "2010-01-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#"2010-2100/3:10:00-12:00"#, "2019-01-01"),
        schedule! { 10,00 => Open => 12,00 }
    );

    assert_eq!(
        schedule_at!(r#"2010-2100/3:10:00-12:00"#, "2017-01-01"),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(r#"2010-2100/3:10:00-12:00"#, "2018-01-01"),
        schedule! {}
    );

    Ok(())
}

#[test]
fn easter() -> Result<(), Error> {
    assert_eq!(schedule_at!("easter", "2024-03-31"), schedule! {});
    Ok(())
}
