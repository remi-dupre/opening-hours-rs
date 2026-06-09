use alloc::string::ToString;

use crate::error::Result;
use crate::parser::parse;
use crate::tests::ex;

/// These expression can be parsed and displayed again to get the same string.
const DISPLAY_STABLE: &[(&str, u32, &str)] = &[
    // Doesn't add 'Mo-Su' in an additional rule if a year selector is set
    ex!("2020, 2021 10:00-16:00"),
];

/// These expression will be displayed differently after parsing
const DISPLAY_MODIFIED: &[(&str, u32, &str, &str)] = &[
    // Doesn't add 'Mo-Su' in an additional rule if a year selector is set
    ex!("2020, 10:00-16:00", "2020, Mo-Su 10:00-16:00"),
    // Used to display an invalid expression as '/' can only follow ranges
    ex!("1975-1975/7", "1975"),
    ex!("week02-02/7", "week02"),
    // Hide hours when it is null
    ex!("12:00-14:00/01:30", "12:00-14:00/01:30"),
    ex!("12:00-14:00/00:30", "12:00-14:00/30"),
];

#[test]
fn display_stable() -> Result<()> {
    for (file, line, example) in DISPLAY_STABLE {
        assert_eq!(
            parse(example)?.to_string(),
            *example,
            "example should be displayed like parsed expression at {file}:{line}",
        );
    }

    Ok(())
}

#[test]
fn display_modified() -> Result<()> {
    for (file, line, example, expected) in DISPLAY_MODIFIED {
        parse(expected).expect("invalid expected expression");

        assert_eq!(
            parse(example)?.to_string(),
            *expected,
            "unexpected stringified expression at {file}:{line}",
        );
    }

    Ok(())
}
