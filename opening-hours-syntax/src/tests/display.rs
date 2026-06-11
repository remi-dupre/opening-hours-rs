use alloc::string::ToString;

use rstest::rstest;

use crate::parser::parse;

#[rstest]
#[case("2020, 2021 10:00-16:00")]
// Offset on variable times require parenthesis to avoid abiguity
#[case("(sunrise-00:10)-(sunset+01:15)")]
#[case("(sunrise-00:10)-sunset")]
#[case("sunrise-(sunset+01:15)")]
fn display_stable(#[case] example: &str) {
    assert_eq!(
        parse(example).expect("invalid example").to_string(),
        example,
    );
}

#[rstest]
// Doesn't add 'Mo-Su' in an additional rule if a year selector is set
#[case("2020, 10:00-16:00", "2020, Mo-Su 10:00-16:00")]
// Used to display an invalid expression as '/' can only follow ranges
#[case("1975-1975/7", "1975")]
#[case("week02-02/7", "week02")]
// Hide hours when it is null
#[case("12:00-14:00/01:30", "12:00-14:00/01:30")]
#[case("12:00-14:00/00:30", "12:00-14:00/30")]
fn display_modified(#[case] example: &str, #[case] expected: &str) {
    // Ensure expected is already stable
    let expected_parsed = parse(expected).expect("invalid expected expression");
    assert_eq!(expected_parsed.to_string(), expected);

    // The test itself
    assert_eq!(
        parse(example).expect("invalid example").to_string(),
        *expected,
    );
}
