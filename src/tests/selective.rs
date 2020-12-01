use crate::parser::Error;
use crate::time_domain::RuleKind::*;
use crate::{datetime, parse, schedule_at};

#[test]
fn s000_idunn_interval_stops_next_day() -> Result<(), Error> {
    use crate::time_domain::DateTimeRange;
    use chrono::Duration;

    let oh = parse("Tu-Su 09:30-18:00; Th 09:30-21:45")?;
    let start = datetime!("2018-06-11 00:00");
    let end = start + Duration::days(1);

    assert_eq!(
        oh.iter_range(start, end).collect::<Vec<_>>(),
        vec![DateTimeRange {
            range: start..end,
            kind: Closed,
            comments: vec![],
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
    assert!(parse("Jan-Feb 10:00-20:00").is_ok());
}

#[test]
fn s004_idunn_until_midnight_as_00() -> Result<(), Error> {
    assert_eq!(
        schedule_at!("Mo-Su 09:00-00:00 open", "2018-06-14"),
        schedule! { 9,00 => Open => 24,00 }
    );

    Ok(())
}
