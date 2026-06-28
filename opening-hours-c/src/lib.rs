pub(crate) mod ctype;
pub(crate) mod util;

#[cfg(test)]
mod tests;

use std::ffi::{CStr, c_char};
use std::str::FromStr;

use opening_hours::OpeningHours;

use crate::util::read_timestamp;

/// Parse an opening hours expression and store the result in the given pointer.
///
/// You will have to free the created object by calling `oh_free`.
///
/// # Inputs
///
/// - `expression`: an utf8 string holding an opening hours expression.
/// - `oh_ptr`: a pointer to the pointer the output should be stored at.
///
/// # Return
///
/// If the parsing succeeds, this function returns 0 and sets *oh_ptr to a
/// valid pointer to an `OH`.
///
/// If the parsing fails, this function returns an error < 0.
///
/// # Safety
///
/// 1. `expression` must be a valid pointer to a null terminated string.
///    See https://doc.rust-lang.org/stable/std/ffi/struct.CStr.html#safety
/// 2. `oh_ptr` must be a valid pointer to an OH pointer. You do not have to
///    allocate data ahead and should not call this on an already parsed OH
///    object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oh_parse(
    expression: *const c_char,
    oh_ptr: *mut *mut ctype::OH,
) -> ctype::OHParserError {
    if expression.is_null() || oh_ptr.is_null() {
        return ctype::OHParserError::NullPtr;
    }

    // Safety: the caller MUST provide a valid a c string pointer.
    let expr_c = unsafe { CStr::from_ptr(expression) };

    let Ok(expr_str) = expr_c.to_str() else {
        return ctype::OHParserError::InvalidUtf8;
    };

    let oh = match OpeningHours::from_str(expr_str) {
        Ok(oh) => oh,
        Err(err) => return err.into(),
    };

    let res_as_ptr = Box::into_raw(Box::new(ctype::OH::from(oh)));

    // Safety: the caller MUST provide a valid reference to a pointer value.
    unsafe { oh_ptr.write(res_as_ptr) }
    ctype::OHParserError::Ok
}

/// Release memory allocated for this parsed opening hours expression.
///
/// This will invalidate all pointers created from it, be sure to copy all data
/// that you would use later (for instance comments)
///
/// # Inputs
///
/// - `oh`: a valid pointer to a parsed expression.
///
/// # Safety
///
/// `oh` must be a valid OH pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oh_free(oh: *mut ctype::OH) {
    if oh.is_null() {
        return;
    }

    // Safety: the call MUST provide a valid pointer
    let oh = unsafe { Box::from_raw(oh) };
    drop(oh)
}

/// Get a string representation of this parsed expression.
///
/// This will output the original expression in a format that is guaranteed to
/// conform to the official specification, but if you want to simplify the
/// expression you may be interested in `oh_normalize`.
///
/// # Inputs
///
/// - `oh`: a valid pointer to a parsed expression.
///
/// # Safety
///
/// 1. `oh` must be a valid OH pointer.
/// 2. The output pointer is valid as long as oh is not destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oh_format(oh: *mut ctype::OH) -> *const c_char {
    // Safety: the caller MUST provide a valid pointer
    let Some(oh) = (unsafe { oh.as_mut() }) else {
        return c"null".as_ptr();
    };

    oh.format().as_ptr()
}

/// Normalize the input expression.
///
/// This will attempt to merge intersecting intervals and will transform the
/// expression in an usually more readable format.
///
/// # Inputs
///
/// - `oh`: a valid pointer to a parsed expression.
///
/// # Safety
///
/// 1. `oh` must be a valid OH pointer.
/// 2. The original expression will be destroyed, all pointer built from it
///    may be invalidated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oh_normalize(oh: *mut ctype::OH) {
    // Safety: the caller MUST provide a valid pointer
    let Some(oh) = (unsafe { oh.as_mut() }) else {
        return;
    };

    oh.normalize();
}

/// Get the current state from the expression.
///
/// # Inputs
///
/// - `oh`: a valid pointer to a parsed expression.
/// - `dt`: a positive 64-bits signed local timestamp.
///
/// # Returns
///
/// An `OHState` struct which bundles current opening status and a comment
/// string that may be null.
///
/// If the comment string is not null, it will be valid until `oh` is destroyed.
///
/// # Safety
///
/// `oh` must be a valid OH pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oh_state_at(oh: *mut ctype::OH, dt: i64) -> ctype::OHState {
    // Safety: the caller MUST provide a valid pointer
    let Some(oh) = (unsafe { oh.as_mut() }) else {
        return ctype::OHState {
            kind: ctype::OHRuleKind::Unknown,
            comment: c"null".as_ptr(),
        };
    };

    let Some(dt) = read_timestamp(dt) else {
        return ctype::OHState {
            kind: ctype::OHRuleKind::Unknown,
            comment: c"invalid timestamp".as_ptr(),
        };
    };

    oh.state_at(dt)
}

/// Get the current state from the expression.
///
/// # Inputs
///
/// - `oh`: a valid pointer to a parsed expression.
/// - `dt`: a positive 64-bits signed local timestamp.
///
/// # Returns
///
/// 1. A positive unsigned 64-bits timestamp if a state change will happen.
/// 2. 0 if no state change will happen.
/// 3. -1 if the provided timestamp or pointer is invalid.
///
/// # Safety
///
/// `oh` must be a valid OH pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oh_next_change(oh: *const ctype::OH, dt: i64) -> i64 {
    // Safety: the caller MUST provide a valid pointer
    let Some(oh) = (unsafe { oh.as_ref() }) else {
        return -1;
    };

    let Some(dt) = read_timestamp(dt) else {
        return -1;
    };

    oh.next_change(dt)
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(0)
}

/// Get a string representation of the state rule kind variants.
///
/// # Inputs
///
/// - `rule_kind`: the value that has to be stringified.
///
/// # Returns
///
/// A static string representing input rule kind.
#[unsafe(no_mangle)]
pub extern "C" fn oh_rule_kind_format(rule_kind: ctype::OHRuleKind) -> *const i8 {
    match rule_kind {
        ctype::OHRuleKind::Open => c"open",
        ctype::OHRuleKind::Closed => c"closed",
        ctype::OHRuleKind::Unknown => c"unknown",
    }
    .as_ptr()
}
