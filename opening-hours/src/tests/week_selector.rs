use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::{datetime, schedule_at, OpeningHours};

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

#[test]
fn last_year_week() -> Result<(), Error> {
    // Week 52 of 7569 is the last week of the year and ends at the 28th
    assert_eq!(
        OpeningHours::parse("week 52 ; Jun")?
            .next_change(datetime!("7569-12-28 08:05"))
            .unwrap(),
        datetime!("7569-12-29 00:00"),
    );

    assert_eq!(
        OpeningHours::parse("week 1 ; Jun")?
            .next_change(datetime!("7569-12-28 08:05"))
            .unwrap(),
        datetime!("7569-12-29 00:00"),
    );

    // Week 52 of 2021 is the last week and ends on the 2th of January
    assert_eq!(
        OpeningHours::parse("week 52 ; Jun")?
            .next_change(datetime!("2021-12-28 08:05"))
            .unwrap(),
        datetime!("2022-01-03 00:00"),
    );

    // Week 53 of 2020 ends on 3rd of January
    assert_eq!(
        OpeningHours::parse("week 53 ; Jun")?
            .next_change(datetime!("2020-12-28 08:05"))
            .unwrap(),
        datetime!("2021-01-04 00:00"),
    );

    // There is no week 53 from 2021 to 2026
    assert_eq!(
        OpeningHours::parse("week 53")?
            .next_change(datetime!("2021-01-15 08:05"))
            .unwrap(),
        datetime!("2026-12-28 00:00"),
    );
    Ok(())
}

#[test]
fn outside_wrapping_range() -> Result<(), Error> {
    let oh = OpeningHours::parse("2030 week52-01")?;
    assert!(oh.next_change(datetime!("2024-06-01 12:00")).is_some());
    assert!(oh.is_closed(datetime!("2024-06-01 12:00")));
    Ok(())
}
