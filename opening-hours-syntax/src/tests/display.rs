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
        assert_eq!(
            parse(example)?.to_string(),
            *expected,
            "unexpected stringified expression at {file}:{line}",
        );
    }

    Ok(())
}
