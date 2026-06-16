use std::ffi::CStr;

use rstest::rstest;

use crate::ctype::{OHParserError, OHRuleKind};
use crate::{oh_format, oh_free, oh_next_change, oh_normalize, oh_parse};

const TS_2020_JUN_06_12H00: i64 = 1591012800;
const TS_2020_JUN_06_18H00: i64 = TS_2020_JUN_06_12H00 + 6 * 60 * 60;

#[rstest]
#[case(c"24/7", OHParserError::Ok)]
#[case(c"247", OHParserError::Syntax)]
#[case(CStr::from_bytes_with_nul(&[255, 0]).unwrap(), OHParserError::InvalidUtf8)]
fn parse(#[case] expr: &CStr, #[case] expected_result: OHParserError) {
    unsafe {
        let mut oh = std::ptr::null_mut();
        let result = oh_parse(expr.as_ptr(), &mut oh);
        assert_eq!(result, expected_result, "wrong parse result for {expr:?}");

        if result == OHParserError::Ok {
            oh_free(oh);
        } else {
            assert!(oh.is_null());
        }
    }
}

#[rstest]
#[case(c"24/7", c"24/7")]
#[case(c"mo-fr 10:00 - 14:00", c"Mo-Fr 10:00-14:00")]
fn format(#[case] expr: &CStr, #[case] expected_format: &CStr) {
    unsafe {
        let mut oh = std::ptr::null_mut();
        assert_eq!(oh_parse(expr.as_ptr(), &mut oh), OHParserError::Ok);
        assert_eq!(CStr::from_ptr(oh_format(oh)), expected_format);
        oh_free(oh);
    }
}

#[rstest]
#[case(c"24/7", c"24/7")]
#[case(c"mo-fr 10:00 - 14:00", c"Mo-Fr 10:00-14:00")]
#[case(c"mo-fr 10:00-14:00,14:00-18:00", c"Mo-Fr 10:00-18:00")]
fn normalize(#[case] expr: &CStr, #[case] expected_format: &CStr) {
    unsafe {
        let mut oh = std::ptr::null_mut();
        assert_eq!(oh_parse(expr.as_ptr(), &mut oh), OHParserError::Ok);
        oh_format(oh); // allocate buffers
        oh_normalize(oh);
        assert_eq!(CStr::from_ptr(oh_format(oh)), expected_format);
        oh_free(oh);
    }
}

#[rstest]
#[case(c"24/7", -1,  OHRuleKind::Unknown, c"invalid timestamp")] // invalid input timestamp
#[case(c"10:00-18:00", TS_2020_JUN_06_12H00, OHRuleKind::Open, c"")]
#[case(cr#""comment""#, TS_2020_JUN_06_12H00, OHRuleKind::Open, c"comment")]
fn state_at(
    #[case] expr: &CStr,
    #[case] timestamp: i64,
    #[case] expected_kind: OHRuleKind,
    #[case] expected_comment: &CStr,
) {
    unsafe {
        use crate::oh_state_at;

        let mut oh = std::ptr::null_mut();
        assert_eq!(oh_parse(expr.as_ptr(), &mut oh), OHParserError::Ok);
        let state = oh_state_at(oh, timestamp);

        assert_eq!(
            state.kind, expected_kind,
            "got wrong state kind for {expr:?} at {timestamp}"
        );

        assert_eq!(
            state.kind, expected_kind,
            "got wrong state comment for {expr:?} at {timestamp}"
        );

        if state.comment.is_null() {
            assert!(
                expected_comment.is_empty(),
                "{expr:?} at {timestamp} should return a comment",
            );
        } else {
            assert_eq!(
                CStr::from_ptr(state.comment),
                expected_comment,
                "got wrong state comment for {expr:?} at {timestamp:?}"
            );
        }

        oh_free(oh);
    }
}

#[rstest]
#[case(c"24/7", -1, -1)] // invalid input timestamp
#[case(c"24/7", TS_2020_JUN_06_12H00, 0)]
#[case(c"10:00-18:00", TS_2020_JUN_06_12H00, TS_2020_JUN_06_18H00)]
fn next_change(#[case] expr: &CStr, #[case] timestamp: i64, #[case] expected_result: i64) {
    unsafe {
        let mut oh = std::ptr::null_mut();
        assert_eq!(oh_parse(expr.as_ptr(), &mut oh), OHParserError::Ok);
        let next_change = oh_next_change(oh, timestamp);

        assert_eq!(
            next_change, expected_result,
            "got wrong next change for {expr:?} at {timestamp}: {next_change} instead of {expected_result}"
        );

        oh_free(oh);
    }
}

#[rstest]
#[case(OHRuleKind::Open, c"open")]
#[case(OHRuleKind::Closed, c"closed")]
#[case(OHRuleKind::Unknown, c"unknown")]
fn test_format_rule_kind(#[case] rule_kind: OHRuleKind, #[case] expected_str: &CStr) {
    unsafe {
        use crate::oh_rule_kind_format;
        assert_eq!(CStr::from_ptr(oh_rule_kind_format(rule_kind)), expected_str)
    }
}
