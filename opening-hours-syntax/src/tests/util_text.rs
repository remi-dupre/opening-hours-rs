use rstest::rstest;

use crate::util::text::{is_capitalized, is_lowercase};

#[rstest]
#[case(true, "")]
#[case(true, "Mo")]
#[case(true, "I am groot 🌱")]
#[case(false, "MO")]
#[case(false, "mo")]
#[case(false, "I Am Groot")]
fn text_is_capitalized(#[case] expected: bool, #[case] example: &str) {
    assert_eq!(
        is_capitalized(example),
        expected,
        "is_capitalized({example:?}) != {expected}",
    )
}

#[rstest]
#[case(true, "")]
#[case(true, "mo")]
#[case(true, "i am groot 🌱")]
#[case(false, "Mo")]
#[case(false, "mO")]
#[case(false, "I Am Groot")]
fn text_is_lowercase(#[case] expected: bool, #[case] example: &str) {
    assert_eq!(
        is_lowercase(example),
        expected,
        "is_lowercase({example:?}) != {expected}",
    )
}
