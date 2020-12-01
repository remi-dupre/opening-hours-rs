use crate::datetime;
use crate::parse;
use crate::parser::Error;
use crate::time_domain::RuleKind::*;

#[test]
fn s000_idunn_empty_interval() -> Result<(), Error> {
    use crate::time_domain::DateTimeRange;
    use chrono::Duration;

    let oh = parse("Tu-Su 09:30-18:00; Th 09:30-21:45")?;
    let start = datetime!("2018-06-11 00:00");
    let end = start + Duration::days(1);

    assert_eq!(
        Some(DateTimeRange {
            range: start..end,
            kind: Closed,
            comments: vec![],
        }),
        oh.intervals(start, end).next()
    );

    Ok(())
}
