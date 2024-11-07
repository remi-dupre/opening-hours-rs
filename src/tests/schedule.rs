use std::sync::Arc;

use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::RuleKind;

use crate::schedule::{Schedule, TimeRange};

#[test]
fn test_filled_iter_on_empty_schedule() {
    let mut intervals = Schedule::default().into_iter_filled();

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(0, 0)..ExtendedTime::new(24, 0),
            kind: RuleKind::Closed,
            comments: Default::default(),
        })
    );

    assert_eq!(intervals.next(), None);
}

#[test]
fn test_filled_iter_on_complex_schedule() {
    let mut intervals = {
        Schedule::from_ranges(
            [
                ExtendedTime::new(10, 0)..ExtendedTime::new(12, 0),
                ExtendedTime::new(14, 0)..ExtendedTime::new(16, 0),
            ],
            RuleKind::Open,
            &vec![Arc::<str>::from("Full availability")].into(),
        )
        .addition(Schedule::from_ranges(
            [ExtendedTime::new(16, 0)..ExtendedTime::new(18, 0)],
            RuleKind::Unknown,
            &Default::default(),
        ))
        .addition(Schedule::from_ranges(
            [ExtendedTime::new(9, 0)..ExtendedTime::new(10, 0)],
            RuleKind::Closed,
            &vec![Arc::<str>::from("May take orders")].into(),
        ))
        .addition(Schedule::from_ranges(
            [ExtendedTime::new(22, 0)..ExtendedTime::new(24, 0)],
            RuleKind::Closed,
            &Default::default(),
        ))
        .into_iter_filled()
    };

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(0, 0)..ExtendedTime::new(10, 0),
            kind: RuleKind::Closed,
            comments: vec![Arc::<str>::from("May take orders")].into(),
        })
    );

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(10, 0)..ExtendedTime::new(12, 0),
            kind: RuleKind::Open,
            comments: vec![Arc::<str>::from("Full availability")].into(),
        })
    );

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(12, 0)..ExtendedTime::new(14, 0),
            kind: RuleKind::Closed,
            comments: Default::default(),
        })
    );

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(14, 0)..ExtendedTime::new(16, 0),
            kind: RuleKind::Open,
            comments: vec![Arc::<str>::from("Full availability")].into(),
        })
    );

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(16, 0)..ExtendedTime::new(18, 0),
            kind: RuleKind::Unknown,
            comments: Default::default(),
        })
    );

    assert_eq!(
        intervals.next(),
        Some(TimeRange {
            range: ExtendedTime::new(18, 0)..ExtendedTime::new(24, 0),
            kind: RuleKind::Closed,
            comments: Default::default(),
        })
    );

    assert_eq!(intervals.next(), None);
}
