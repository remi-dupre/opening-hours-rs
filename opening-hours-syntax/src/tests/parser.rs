use rstest::rstest;

use crate::rules::OpeningHoursExpression;

#[rstest]
#[case("12:00+")]
#[case("24/7")]
#[case("dusk-dusk+")]
#[case("Jun24:00+")]
#[case("10:00-18:00+")]
#[case("10:00-18:00/30")]
#[case("10:00-18:00/01:30")]
#[case(r#"Mo-Fr open "ring the bell""#)]
#[case("Jan Mo[1]-30")]
// Comments
#[case::comment(r#"open "comment""#)]
#[case::comment(r#""comment""#)]
#[case::comment(r#"24/7 "comment""#)]
// Ambiguity of a weekday after a month, it could either be a full montday
// range or a monthday range followed by a weekday range.
#[case("Feb Mo[1] +2 days")]
#[case("Feb Mo[1],Tu-Fr")]
// Expressions that are not handled by the documented grammar but either make sense or are commonly
// supported by other libraries
#[case::relaxed("4:00-8:00")]
#[case::relaxed("04:00 - 08:00")]
#[case::relaxed("4:00 - 8:00")]
#[case::relaxed("Mo-Fr 10:00-18:00;Sa-Su 10:00-12:00")]
fn parse_valid(#[case] expression: &str) {
    assert!(
        str::parse::<OpeningHoursExpression>(expression).is_ok(),
        "parser fails on {expression:?}",
    );
}

#[rstest]
#[case("Parser", "not a valid expression")]
#[case("Parser", "10:00-100:00")]
#[case("Parser", "10:00-12:00 tomorrow")]
#[case("Parser", r#"Mo-Fr open "ring "the bell"""#)]
#[case("InvertedWeekRange", "week15-10")]
#[case("InvertedWeekRange", "week15-10/3")]
#[case("InvertedYearRange", "2020-2010")]
#[case("InvertedYearRange", "2020-2010/3")]
// Comments
#[case::comment("Parser", r#"open "comment1" "comment2""#)]
#[case::comment("Parser", r#""comment" open"#)] // could be allowed with warning
// 00 is invalid for monthdays
#[case::monthday00("Parser", "Jan 0")]
#[case::monthday00("Parser", "Jan 00")]
#[case::monthday00("Parser", "Jan 00-15")]
// Extended time (>24h) can't start a time range
#[case::extended_start("Parser", "27:43+")]
#[case::extended_start("Parser", "24:11+")]
#[case::extended_start("Parser", "27:43-28:00")]
#[case::extended_start("Parser", "24:11-28:00")]
#[case::extended_start("Parser", "27:43-10:00")]
fn parse_invalid(#[case] expected_error_variant: &str, #[case] expression: &str) {
    let Err(err) = str::parse::<OpeningHoursExpression>(expression) else {
        panic!("parser should have raised an error on {expression}")
    };

    let err_debug = format!("{err:?}");

    let err_variant = err_debug
        .split_once([' ', '('])
        .map(|(variant, _)| variant)
        .unwrap_or(&err_debug);

    assert_eq!(err_variant, expected_error_variant);
}
