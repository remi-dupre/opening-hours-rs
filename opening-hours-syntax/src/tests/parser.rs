use alloc::string::{String, ToString};
use alloc::vec::Vec;

use rstest::rstest;

use crate::tests::parser_no_warn;
use crate::Parser;

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
fn parse_valid(mut parser_no_warn: Parser, #[case] expr: &str) {
    assert!(
        parser_no_warn.parse(expr).is_ok(),
        "parser fails on {expr:?}",
    );
}

#[rstest]
#[case("Syntax", "not a valid expression")]
#[case("Syntax", "10:00-100:00")]
#[case("Syntax", "10:00-12:00 tomorrow")]
#[case("Syntax", r#"Mo-Fr open "ring "the bell"""#)]
#[case("InvertedWeekRange", "week15-10")]
#[case("InvertedWeekRange", "week15-10/3")]
#[case("InvertedYearRange", "2020-2010")]
#[case("InvertedYearRange", "2020-2010/3")]
#[case("Overflow", "week 1-50/300")]
#[case("InvalidExtendedTime", "00:00-48:01")]
#[case("InvertedYearRange", "2020-2010")]
#[case("InvertedWeekRange", "week 50-10")]
// Comments
#[case::comment("Syntax", r#"open "comment1" "comment2""#)]
#[case::comment("Syntax", r#""comment" open"#)] // could be allowed with warning
// 00 is invalid for monthdays
#[case::monthday00("Syntax", "Jan 0")]
#[case::monthday00("Syntax", "Jan 00")]
#[case::monthday00("Syntax", "Jan 00-15")]
// Extended time (>24h) can't start a time range
#[case::extended_start("Syntax", "27:43+")]
#[case::extended_start("Syntax", "24:11+")]
#[case::extended_start("Syntax", "27:43-28:00")]
#[case::extended_start("Syntax", "24:11-28:00")]
#[case::extended_start("Syntax", "27:43-10:00")]
fn parse_invalid(
    mut parser_no_warn: Parser,
    #[case] expected_error_variant: &str,
    #[case] expr: &str,
) {
    let Err(err) = parser_no_warn.parse(expr) else {
        panic!("parser should have raised an error on {expr}")
    };

    let err_debug = format!("{err:?}");

    let err_variant = err_debug
        .split_once([' ', '('])
        .map(|(variant, _)| variant)
        .unwrap_or(&err_debug);

    assert_eq!(err_variant, expected_error_variant);

    assert!(
        !err.is_implementation_error(),
        "{err_variant} should not be marked to be an implementation error",
    );
}

#[rstest]
// Weekday
#[case::wday("mo-fr", &["ShouldBeCapitalized"])]
#[case::wday("mO-fR", &["ShouldBeCapitalized"])]
#[case::wday("Mo-FR", &["ShouldBeCapitalized"])]
// Monthday
#[case::monthday("jan-mar", &["ShouldBeCapitalized"])]
#[case::monthday("Jan-mar", &["ShouldBeCapitalized"])]
#[case::monthday("Jan-MAR", &["ShouldBeCapitalized"])]
#[case::monthday("Easter-Mar 31", &["ShouldBeLowercase"])]
// Rule Modifier
#[case::modifier("OPEN", &["ShouldBeLowercase"])]
#[case::modifier("Off", &["ShouldBeLowercase"])]
#[case::modifier("closeD", &["ShouldBeLowercase"])]
#[case::modifier("UNKNOWN", &["ShouldBeLowercase"])]
// Event
#[case::event("DAWN-DUSK", &["ShouldBeLowercase"])]
#[case::event("Dawn-Dusk", &["ShouldBeLowercase"])]
// Mixed
#[case::mix("Jan mo-tu OPEN", &["ShouldBeCapitalized", "ShouldBeLowercase"])]
fn parse_with_warnings(#[case] expr: &str, #[case] expected_warnings: &[&str]) {
    let mut warnings: Vec<String> = Vec::default();

    let mut parser = Parser::default().with_warning_handler(|warning| {
        let warn_debug = format!("{warning:?}");

        let err_variant = warn_debug
            .split_once([' ', '('])
            .map(|(variant, _)| variant)
            .unwrap_or(&warn_debug);

        warnings.push(err_variant.to_string());
    });

    assert!(parser.parse(expr).is_ok(), "{expr} should be valid");
    warnings.sort();
    warnings.dedup();

    let expected_warnings: Vec<String> = expected_warnings
        .iter()
        .copied()
        .map(str::to_string)
        .collect();

    assert_eq!(
        warnings, expected_warnings,
        "warnings are not as expected for {expr}"
    );
}
