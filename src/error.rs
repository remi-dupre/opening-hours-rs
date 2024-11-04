#[derive(thiserror::Error, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[error("Unknown ISO code `{0}`")]
pub struct UnknownCountryCode(pub String);
