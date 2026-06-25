use std::ops::Range;

use chrono::NaiveDate;
use opening_hours_syntax::extended_time::ExtendedTime;
use opening_hours_syntax::rules::RuleKind::*;
use rstest::rstest;

use crate::localization::{
    Country,
    Country::{DE, FR, US},
};
use crate::schedule::{Schedule, TimeRange};
use crate::tests::utils::parse::xt;
use crate::{Context, OpeningHours};

#[rstest]
#[case("2020-06-01", "24/7", "00:00 open 24:00")]
// Time Span
#[case::timespan("2020-06-01", "14:00-19:00", "14:00 open 19:00")]
#[case::timespan("2020-06-01", "Mo 14:00-19:00", "14:00 open 19:00")]
#[case::timespan("2020-06-01", "Su 14:00-19:00", Schedule::new())]
#[case::timespan("2020-06-01", "Su 14:00-25:30", "00:00 open 01:30")]
#[case::timespan("2020-06-01", "10:00-12:00,11:00-16:00 unknown", "10:00 unknown 16:00")]
#[case::timespan("2020-06-01", "23:00-01:00", "00:00 open 01:00 | 23:00 open 24:00")]
#[case::timespan("2025-02-23", "Mo 04:00-04:00", Schedule::new())] // sunday
#[case::timespan("2025-02-24", "Mo 04:00-04:00", "04:00 open 24:00")] // monday
#[case::timespan("2025-02-25", "Mo 04:00-04:00", "00:00 open 04:00")] // tuesday
#[case::timespan("2025-02-26", "Mo 04:00-04:00", Schedule::new())] // wednesday
#[case::timespan(
    "2020-06-01",
    "10:00-12:00,14:00-16:00",
    "10:00 open 12:00 | 14:00 open 16:00"
)]
#[case::timespan(
    "2020-06-01",
    "10:00-12:00,14:00-25:30",
    "00:00 open 01:30 | 10:00 open 12:00 | 14:00 open 24:00"
)]
// Weekday Range
#[case::weekday("2020-06-01", "Mo-Su", "00:00 open 24:00")]
#[case::weekday("2020-06-02", "Tu", "00:00 open 24:00")]
#[case::weekday("2020-06-02", "We", Schedule::new())]
#[case::weekday("2020-06-01", "Mo-Tu,Th,Sa-Su 10:00-12:00", "10:00 open 12:00")]
#[case::weekday("2020-06-02", "Mo-Tu,Th,Sa-Su 10:00-12:00", "10:00 open 12:00")]
#[case::weekday("2020-06-03", "Mo-Tu,Th,Sa-Su 10:00-12:00", Schedule::new())]
#[case::weekday("2020-06-04", "Mo-Tu,Th,Sa-Su 10:00-12:00", "10:00 open 12:00")]
#[case::weekday("2020-06-05", "Mo-Tu,Th,Sa-Su 10:00-12:00", Schedule::new())]
#[case::weekday("2020-06-06", "Mo-Tu,Th,Sa-Su 10:00-12:00", "10:00 open 12:00")]
#[case::weekday("2020-06-07", "Mo-Tu,Th,Sa-Su 10:00-12:00", "10:00 open 12:00")]
// Weekday Range (index)
#[case::weekday_nth("2020-06-01", "Mo[1] 10:00-12:00", "10:00 open 12:00")]
#[case::weekday_nth("2020-06-02", "Mo[1] 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-06-08", "Mo[1] 10:00-12:00", Schedule::new())]
// . Index range
#[case::weekday_nth("2020-06-01", "Mo[2-4] 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-06-08", "Mo[2-4] 10:00-12:00", "10:00 open 12:00")]
#[case::weekday_nth("2020-06-15", "Mo[2-4] 10:00-12:00", "10:00 open 12:00")]
#[case::weekday_nth("2020-06-22", "Mo[2-4] 10:00-12:00", "10:00 open 12:00")]
#[case::weekday_nth("2020-06-29", "Mo[2-4] 10:00-12:00", Schedule::new())]
// . Negative index
#[case::weekday_nth("2020-06-01", "Su[-1] 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-06-02", "Su[-1] 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-06-07", "Su[-1] 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-06-14", "Su[-1] 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-06-21", "Su[-1] 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-06-27", "Su[-1] 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-06-28", "Su[-1] 10:00-12:00", "10:00 open 12:00")]
#[case::weekday_nth("2020-06-29", "Su[-1] 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-06-30", "Su[-1] 10:00-12:00", Schedule::new())]
// . Index with offset
#[case::weekday_nth("2020-06-03", "Mo[2-4] +2 days 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-06-10", "Mo[2-4] +2 days 10:00-12:00", "10:00 open 12:00")]
#[case::weekday_nth("2020-06-17", "Mo[2-4] +2 days 10:00-12:00", "10:00 open 12:00")]
#[case::weekday_nth("2020-06-24", "Mo[2-4] +2 days 10:00-12:00", "10:00 open 12:00")]
#[case::weekday_nth("2020-07-01", "Mo[2-4] +2 days 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-05-31", "Mo[1] -1 days 10:00-12:00", "10:00 open 12:00")]
#[case::weekday_nth("2020-06-01", "Mo[1] -1 days 10:00-12:00", Schedule::new())]
#[case::weekday_nth("2020-06-07", "Mo[1] -1 days 10:00-12:00", Schedule::new())]
// Week Range
#[case::week("2020-01-01", r#"week01:10:00-12:00"#, "10:00 open 12:00")]
#[case::week("2020-01-06", r#"week01:10:00-12:00"#, Schedule::new())]
#[case::week("2020-01-06", r#"week01,23-24:10:00-12:00"#, Schedule::new())]
#[case::week("2020-05-31", r#"week01,22-23:10:00-12:00"#, "10:00 open 12:00")]
#[case::week("2020-06-07", r#"week01,22-23:10:00-12:00"#, "10:00 open 12:00")]
#[case::week("2020-01-01", r#"week01-53/2:10:00-12:00"#, "10:00 open 12:00")]
#[case::week("2020-01-08", r#"week01-53/2:10:00-12:00"#, Schedule::new())]
#[case::week("2020-01-15", r#"week01-53/2:10:00-12:00"#, "10:00 open 12:00")]
#[case::week("2020-01-22", r#"week01-53/2:10:00-12:00"#, Schedule::new())]
#[case::week("2020-01-29", r#"week01-53/2:10:00-12:00"#, "10:00 open 12:00")]
// Month Selector (range)
#[case::month("2020-05-31", r#"2020Jun01 open"#, Schedule::new())]
#[case::month("2020-06-01", r#"2020Jun01:10:00-12:10"#, "10:00 open 12:10")]
#[case::month("2020-06-02", r#"2020Jun01 open"#, Schedule::new())]
#[case::month("2020-06-01", r#"Jan-Jun:11:58-11:59"#, "11:58 open 11:59")]
#[case::month("2020-06-01", r#"May15-01:10:00-12:00"#, "10:00 open 12:00")]
#[case::month("2020-06-02", r#"May15-01:10:00-12:00"#, Schedule::new())]
#[case::month("2020-06-01", r#"2019Sep01-2020Jul31:10:00-12:00"#, "10:00 open 12:00")]
#[case::month("2020-06-01", r#"2019Sep01+:10:00-12:00"#, "10:00 open 12:00")]
#[case::month("2020-06-01", r#"2019Sep01-Jul01:10:00-12:00"#, "10:00 open 12:00")]
#[case::month("2020-06-01", r#"Sep01-Jul01:10:00-12:00"#, "10:00 open 12:00")]
#[case::month(
    "2020-01-01",
    r#"open "comment"; (sunset-00:30)-(sunrise-00:15) closed; Mar01-Sep30 (sunset-00:30)-07:30 closed"#,
    "06:45 open[comment] 18:30"
)]
#[case::month(
    "2020-06-01",
    r#"open "comment"; (sunset-00:30)-(sunrise-00:15) closed; Mar01-Sep30 (sunset-00:30)-07:30 closed"#,
    "07:30 open[comment] 18:30"
)]
// Month Selector (out of february bounds)
#[case::month_oob("2020-01-31", "Feb01-Feb31:10:00-12:00", Schedule::new())]
#[case::month_oob("2020-02-01", "Feb01-Feb31:10:00-12:00", "10:00 open 12:00")]
#[case::month_oob("2020-02-28", "Feb01-Feb31:10:00-12:00", "10:00 open 12:00")]
#[case::month_oob("2020-02-29", "Feb01-Feb31:10:00-12:00", "10:00 open 12:00")]
#[case::month_oob("2020-03-01", "Feb01-Feb31:10:00-12:00", Schedule::new())]
#[case::month_oob("2021-03-01", "Feb01-Feb31:10:00-12:00", Schedule::new())]
// Month Selector (with weekday)
#[case::month_wday("2020-01-01", "Feb Mo[2]-Sep Su[-1] 10:00-12:00", Schedule::new())]
#[case::month_wday("2020-06-01", "Feb Mo[2]-Sep Su[-1] 10:00-12:00", "10:00 open 12:00")]
// Year Selector
#[case::year("2020-01-01", "2020:10:00-12:00", "10:00 open 12:00")]
#[case::year("2021-01-01", "2020:10:00-12:00", Schedule::new())]
#[case::year("2020-01-01", "2010-2019,2021,2025+:10:00-12:00", Schedule::new())]
#[case::year("2024-01-01", "2010-2019,2021,2025+:10:00-12:00", Schedule::new())]
#[case::year("2015-01-01", "2010-2019,2021,2025+:10:00-12:00", "10:00 open 12:00")]
#[case::year("5742-01-01", "2010-2019,2021,2025+:10:00-12:00", "10:00 open 12:00")]
#[case::year("2010-01-01", "2010-2100/3:10:00-12:00", "10:00 open 12:00")]
#[case::year("2019-01-01", "2010-2100/3:10:00-12:00", "10:00 open 12:00")]
#[case::year("2017-01-01", "2010-2100/3:10:00-12:00", Schedule::new())]
#[case::year("2018-01-01", "2010-2100/3:10:00-12:00", Schedule::new())]
// Time Events
#[case::events("2020-06-01", "(dawn-02:30)-(dusk+02:30)", "03:30 open 22:30")]
#[case::events("2020-06-01", "(dawn+00:30)-(dusk-00:30)", "06:30 open 19:30")]
#[case::events("2020-06-01", "sunrise-19:45", "07:00 open 19:45")]
#[case::events("2020-06-01", "08:15-sunset", "08:15 open 19:00")]
// Rule: Normal
#[case::rule_normal("2020-06-01", "Jun ; 00:00-04:00 closed", "04:00 open 24:00")]
#[case::rule_normal("2020-07-01", "Jun ; 00:00-04:00 closed", Schedule::new())]
#[case::rule_normal("2181-08-01", "Tu closed; Jul sunset-15:19+", "00:00 open 15:19")]
#[case::rule_normal(
    "2020-06-01",
    "Sa,Su 11:00-13:45 open; 10:00-18:00",
    "10:00 open 18:00"
)]
#[case::rule_normal(
    "2020-05-31",
    "Sa,Su 11:00-13:45 open; 10:00-18:00",
    "10:00 open 18:00"
)]
// Rule: Addional
#[case::rule_additional(
    "2020-06-01",
    "10:00-12:00 open, 14:00-16:00 unknown, 16:00-23:00 closed",
    "10:00 open 12:00 | 14:00 unknown 16:00"
)]
#[case::rule_additional(
    "2020-06-01",
    "10:00-20:00 open, 12:00-14:00 closed",
    "10:00 open 12:00 | 14:00 open 20:00"
)]
#[case::rule_additional(
    "2020-06-01",
    "12:00-14:00 closed, 10:00-20:00 open",
    "10:00 open 20:00"
)]
// Rule: Fallback
#[case::rule_fallback("2020-06-01", "Jun:10:00-12:00 open || unknown", "10:00 open 12:00")]
#[case::rule_fallback("2020-05-31", "Jun:10:00-12:00 open || unknown", "00:00 unknown 24:00")]
#[case::rule_fallback(
    "2020-06-01",
    "Jun:10:00-12:00 open || Mo-Fr closed || unknown",
    "10:00 open 12:00"
)]
#[case::rule_fallback(
    "2020-05-29", // friday
    "Jun:10:00-12:00 open || Mo-Fr closed || unknown",
    "00:00 unknown 24:00"
)]
#[case::rule_fallback(
    "2020-05-30", // saturday
    "Jun:10:00-12:00 open || Mo-Fr closed || unknown",
    "00:00 unknown 24:00"
)]
// Rules with comments (may overlap)
#[case::comment(
    "2020-06-01",
    r#"10:00-12:00 open "welcome!""#,
    "10:00 open[welcome!] 12:00"
)]
#[case::comment(
    "2024-01-01",
    r#"10:00-18:00 "may close later" ; 12:00-13:00 closed "ring the bell""#,
    "10:00 open[may close later] 12:00 closed[ring the bell] 13:00 open[may close later] 18:00"
)]
#[case::comment(
    "2024-01-01",
    r#"10:00-18:00 "may close later", 12:00-13:00 "ring the bell""#,
    "10:00 open[may close later] 12:00 open[ring the bell] 13:00 open[may close later] 18:00"
)]
#[case::comment(
    "2024-01-01",
    r#"10:00-14:00 "may open earlier", 14:00-18:00 "may close later""#,
    "10:00 open[may open earlier] 14:00 open[may close later] 18:00"
)]
fn schedule_at(
    #[case] date: NaiveDate,
    #[case] expr: OpeningHours,
    #[case] expected_schedule: Schedule,
) {
    assert_eq!(
        expr.schedule_at(date),
        expected_schedule,
        "schedule for {expr} at {date} differs from expected",
    );
}

#[cfg(feature = "auto-timezone")]
#[rstest]
#[case("2020-06-01", "sunrise-19:45", "05:51 open 19:45")]
#[case("2020-06-01", "08:15-sunset", "08:15 open 21:46")]
#[case("2020-06-01", "(dawn+00:30)-(dusk-00:30)", "05:40 open 21:57")]
#[case(
    "2020-06-01",
    "(dawn-02:30)-(dusk+02:30)",
    "00:00 open 00:56 | 02:40 open 24:00"
)]
fn schedule_at_with_timezone(
    #[case] date: NaiveDate,
    #[case] expr: OpeningHours,
    #[case] expected_schedule: Schedule,
) {
    use crate::localization::{Coordinates, TzLocation};

    let coords = Coordinates::new(48.87, 2.29).unwrap();
    let ctx = Context::default().with_locale(TzLocation::from_coords(coords));
    let expr = expr.with_context(ctx);

    assert_eq!(
        expr.schedule_at(date),
        expected_schedule,
        "schedule for {expr} at {date} {coords} differs from expected",
    );
}

#[rstest]
// The 14th of July is a holiday in France, not in the US
#[case(FR, "2020-07-14", "10:00-12:00; PH off", Schedule::new())]
#[case(US, "2020-07-14", "10:00-12:00; PH off", "10:00 open 12:00")]
// Independence Day is a federal holiday. If July 4 is a Saturday, it is
// observed on Friday, July 3.
#[case(US, "2020-07-03", "10:00-12:00; PH off", Schedule::new())]
#[case(US, "2020-07-04", "10:00-12:00; PH off", "10:00 open 12:00")]
// International Women's Day is only a regional holiday in Berlin for Germany
#[case(DE, "2025-03-08", "10:00-12:00; PH off", "00:00 unknown 24:00")]
#[case(
    DE,
    "2025-03-08",
    "08:00-18:00, PH 12:00-14:00 off",
    "08:00 open 12:00 unknown 14:00 open 18:00"
)]
fn schedule_at_with_country(
    #[case] country: Country,
    #[case] date: NaiveDate,
    #[case] expr: OpeningHours,
    #[case] expected_schedule: Schedule,
) {
    let ctx = Context::default()
        .with_holidays(country.holidays())
        .with_holidays_unknown(country.holidays_regional());

    let expr = expr.with_context(ctx);

    assert_eq!(
        expr.schedule_at(date),
        expected_schedule,
        "schedule for {expr} at {country} {date} differs from expected",
    );
}

#[rstest]
#[case(&[], Schedule::new())]
#[case(&[xt("00:00")..xt("24:00")], "00:00 Open 24:00")]
#[case(&[xt("12:34")..xt("23:45")], "12:34 Open 23:45")]
#[case(
    &[
        xt("10:00")..xt("12:00"),
        xt("11:00")..xt("14:00"),
        xt("10:30")..xt("11:30"),
        xt("11:00")..xt("14:00"),
        xt("16:00")..xt("20:00"),
        xt("15:00")..xt("19:00"),
        xt("21:00")..xt("21:00"),
    ],
    "10:00 open 14:00 | 15:00 open 20:00",
)]
fn from_ranges(#[case] ranges: &[Range<ExtendedTime>], #[case] expected_schedule: Schedule) {
    assert_eq!(
        Schedule::from_ranges(ranges.to_vec(), Open, "".into()),
        expected_schedule,
    );
}

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
