use pest::iterators::Pair;

use crate::parser::Rule;

// --
// -- Util: text
// --

pub(crate) fn is_capitalized(s: &str) -> bool {
    let mut chars = s.chars();

    let Some(first_char) = chars.next() else {
        return true;
    };

    first_char.is_uppercase() && chars.all(|c| c.is_lowercase())
}

pub(crate) fn is_lowercase(s: &str) -> bool {
    s.chars().all(|c| c.is_lowercase())
}

// --
// -- Enum: Sign
// --

pub(crate) enum Sign {
    Neg,
    Pos,
}

// --
// -- Trait: PairsIterExtension
// --

/// Extra helpers for pairs iterator.
pub(crate) trait PairsIterExtension {
    // Pairs iterator behaves like a std::iter::Peakable but lacks a next_if method.
    fn next_if_rule(&mut self, rule: Rule) -> Option<Pair<'_, Rule>>;
}

impl PairsIterExtension for pest::iterators::Pairs<'_, Rule> {
    fn next_if_rule(&mut self, rule: Rule) -> Option<Pair<'_, Rule>> {
        let pair = self.peek()?;

        if pair.as_rule() == rule {
            self.next()
        } else {
            None
        }
    }
}
