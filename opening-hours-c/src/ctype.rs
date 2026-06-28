// --
// -- OH
// --

use std::ffi::{CStr, CString, c_char};
use std::{collections::HashMap, ops::Deref, sync::Arc};

use chrono::NaiveDateTime;
use opening_hours::{OpeningHours, RuleKind};

use crate::util::string_into_c_lossy;

pub struct OH {
    inner: opening_hours::OpeningHours,
    // Keep ownership of cached C type conversions
    cache_format: Option<CString>,
    cache_comments: HashMap<Arc<str>, CString>,
}

impl OH {
    pub(crate) fn state_at(&mut self, dt: NaiveDateTime) -> OHState {
        let (kind, comment) = self.inner.state(dt);

        let comment = self
            .cache_comments
            .entry(comment.clone())
            .or_insert_with(|| string_into_c_lossy(comment.to_string()));

        let comment = {
            if comment.is_empty() {
                std::ptr::null()
            } else {
                comment.as_ptr()
            }
        };

        OHState { kind: kind.into(), comment }
    }

    pub(crate) fn normalize(&mut self) {
        let Self { inner, cache_comments, .. } = self;
        *self = Self {
            inner: inner.normalize(),
            cache_format: None,
            cache_comments: std::mem::take(cache_comments),
        };
    }

    pub(crate) fn format(&mut self) -> &CStr {
        self.cache_format
            .get_or_insert_with(|| string_into_c_lossy(self.inner.to_string()))
            .as_c_str()
    }
}

impl From<opening_hours::OpeningHours> for OH {
    fn from(value: opening_hours::OpeningHours) -> Self {
        Self {
            inner: value,
            cache_format: None,
            cache_comments: HashMap::new(),
        }
    }
}

impl Deref for OH {
    type Target = OpeningHours;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// --
// -- OHParserError
// --

/// Error type as a result of parsing an opening hours expression.
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum OHParserError {
    /// The expression parsed correctly
    Ok = 0,
    /// The parser was given a null pointer
    NullPtr = -1,
    /// The given expression contains invalid UTF-8
    InvalidUtf8 = -2,
    /// Syntax error
    Syntax = -3,
    /// This error should never be raised. Please report a bug at
    /// https://github.com/remi-dupre/opening-hours-rs/issues
    Implementation = -4,
}

impl From<opening_hours_syntax::Error> for OHParserError {
    fn from(value: opening_hours_syntax::Error) -> Self {
        if value.is_implementation_error() {
            Self::Implementation
        } else {
            Self::Syntax
        }
    }
}

// --
// -- OHKind
// --

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum OHRuleKind {
    Open = 0,
    Closed = 1,
    Unknown = 2,
}

impl From<RuleKind> for OHRuleKind {
    fn from(value: RuleKind) -> Self {
        match value {
            RuleKind::Open => OHRuleKind::Open,
            RuleKind::Closed => OHRuleKind::Closed,
            RuleKind::Unknown => OHRuleKind::Closed,
        }
    }
}

// --
// -- OHState
// --

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct OHState {
    pub kind: OHRuleKind,
    pub comment: *const c_char,
}
