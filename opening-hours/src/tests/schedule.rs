use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::RuleKind::*;
use opening_hours_syntax::Error;

use crate::schedule::{Schedule, TimeRange};
use crate::schedule_at;

#[test]
fn iter_on_empty_schedule() {
    let mut intervals = Schedule::default().into_iter();

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(0, 0).unwrap()..ExtendedTime::new(24, 0).unwrap(),
            kind: Closed,
            comment: Default::default(),
        })
    );

    assert_eq!(intervals.next(), None);
}

#[test]
fn from_ranges() {
    assert_eq!(Schedule::from_ranges([], Open, "".into()), schedule! {});

    assert_eq!(
        Schedule::from_ranges(
            [
                ExtendedTime::new(10, 00).unwrap()..ExtendedTime::new(12, 00).unwrap(),
                ExtendedTime::new(10, 30).unwrap()..ExtendedTime::new(11, 30).unwrap(),
                ExtendedTime::new(11, 00).unwrap()..ExtendedTime::new(14, 00).unwrap(),
                ExtendedTime::new(16, 00).unwrap()..ExtendedTime::new(20, 00).unwrap(),
                ExtendedTime::new(15, 00).unwrap()..ExtendedTime::new(19, 00).unwrap(),
                ExtendedTime::new(21, 00).unwrap()..ExtendedTime::new(21, 00).unwrap(),
            ],
            Open,
            "".into()
        ),
        schedule! {
            10,00 => Open => 14,00;
            15,00 => Open => 20,00;
        }
    );
}

#[test]
fn iter_on_complex_schedule() {
    let mut intervals = {
        Schedule::from_ranges(
            [
                ExtendedTime::new(10, 0).unwrap()..ExtendedTime::new(12, 0).unwrap(),
                ExtendedTime::new(14, 0).unwrap()..ExtendedTime::new(16, 0).unwrap(),
            ],
            Open,
            "Full availability".into(),
        )
        .addition(Schedule::from_ranges(
            [ExtendedTime::new(16, 0).unwrap()..ExtendedTime::new(18, 0).unwrap()],
            Unknown,
            Default::default(),
        ))
        .addition(Schedule::from_ranges(
            [ExtendedTime::new(9, 0).unwrap()..ExtendedTime::new(10, 0).unwrap()],
            Closed,
            "May take orders".into(),
        ))
        .addition(Schedule::from_ranges(
            [ExtendedTime::new(22, 0).unwrap()..ExtendedTime::new(24, 0).unwrap()],
            Closed,
            Default::default(),
        ))
        .into_iter()
    };

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(0, 0).unwrap()..ExtendedTime::new(9, 0).unwrap(),
            kind: Closed,
            comment: "".into(),
        })
    );

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(9, 0).unwrap()..ExtendedTime::new(10, 0).unwrap(),
            kind: Closed,
            comment: "May take orders".into(),
        })
    );

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(10, 0).unwrap()..ExtendedTime::new(12, 0).unwrap(),
            kind: Open,
            comment: "Full availability".into(),
        })
    );

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(12, 0).unwrap()..ExtendedTime::new(14, 0).unwrap(),
            kind: Closed,
            comment: Default::default(),
        })
    );

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(14, 0).unwrap()..ExtendedTime::new(16, 0).unwrap(),
            kind: Open,
            comment: "Full availability".into(),
        })
    );

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(16, 0).unwrap()..ExtendedTime::new(18, 0).unwrap(),
            kind: Unknown,
            comment: Default::default(),
        })
    );

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(18, 0).unwrap()..ExtendedTime::new(24, 0).unwrap(),
            kind: Closed,
            comment: Default::default(),
        })
    );

    assert_eq!(intervals.next(), None);
}

#[test]
fn overlapping_comments() -> Result<(), Error> {
    assert_eq!(
        schedule_at!(
            r#"10:00-18:00 "may close later" ; 12:00-13:00 closed "ring the bell for special orders""#,
            "2024-01-01"
        ),
        schedule! {
               10,00
            => Open, "may close later"
            => 12,00
            => Closed, "ring the bell for special orders"
            => 13,00
            => Open, "may close later"
            => 18,00
        }
    );

    assert_eq!(
        schedule_at!(
            r#"10:00-18:00 "may close later", 12:00-13:00 "ring the bell for special orders""#,
            "2024-01-01"
        ),
        schedule! {
               10,00
            => Open, "may close later"
            => 12,00
            => Open, "ring the bell for special orders"
            => 13,00
            => Open, "may close later"
            => 18,00
        }
    );

    assert_eq!(
        schedule_at!(
            r#"10:00-14:00 "may open earlier", 14:00-18:00 "may close later""#,
            "2024-01-01"
        ),
        schedule! {
               10,00
            => Open, "may open earlier"
            => 14,00
            => Open, "may close later"
            => 18,00
        }
    );

    Ok(())
}
