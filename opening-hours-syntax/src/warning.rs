use pest::iterators::Pair;

use crate::parser::Rule;

/// Some warning emited during the expression parsing. These warning are not
/// critical and the expression will be parsed with no ambuiguity.
#[derive(Debug, Clone)]
pub enum Warning<'e> {
    /// The literal should be capitalized
    ShouldBeCapitalized(Pair<'e, Rule>),
    /// The literal should be lowercase
    ShouldBeLowercase(Pair<'e, Rule>),
}

impl<'e> Warning<'e> {
    // Get the pair that emited this warning.
    pub fn as_pair(&self) -> &Pair<'e, Rule> {
        match self {
            Self::ShouldBeCapitalized(pair) | Self::ShouldBeLowercase(pair) => pair,
        }
    }
}

impl core::fmt::Display for Warning<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ShouldBeCapitalized(pair) => write!(
                f,
                "{:?} literal should be capitalized, got '{}'",
                pair.as_rule(),
                pair.as_str(),
            ),
            Self::ShouldBeLowercase(pair) => write!(
                f,
                "{:?} literal should be lowercase, got '{}'",
                pair.as_rule(),
                pair.as_str(),
            ),
        }
    }
}
