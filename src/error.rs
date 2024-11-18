use std::fmt::Display;

pub use opening_hours_syntax::error::Error as ParserError;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnknownCountryCode(pub String);

impl Display for UnknownCountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown ISO code `{}`", self.0)
    }
}

impl std::error::Error for UnknownCountryCode {}
