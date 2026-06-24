use crate::Parser;

mod display;
mod normalize;
mod parser;
mod paving;
mod rule_time;

/// A parsure that asserts that no warning are emited.
#[rstest::fixture]
fn parser_no_warn() -> Parser {
    Parser::default().with_warning_handler(|warning| {
        panic!(
            "Received an unexpected warning while parsing {}: {warning}",
            warning.as_pair().get_input(),
        )
    })
}
