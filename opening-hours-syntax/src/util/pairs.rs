use pest::iterators::Pair;

use crate::parser::Rule;

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
