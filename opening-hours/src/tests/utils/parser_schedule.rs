use crate::schedule::Schedule;
use opening_hours_syntax::{ExtendedTime, RuleKind};
use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use std::str::FromStr;

#[derive(Parser)]
#[grammar_inline = r#"
    WHITESPACE = _{ " " | "\t" | NEWLINE }

    schedule = {
        SOI ~ chain ~ ("|" ~ chain)* ~ EOI
    }

    chain = {
        time ~ (rule ~ time)+
    }

    rule = {
        ident
        ~ annotation*
    }

    annotation = {
        "[" ~ annotation_text ~ "]"
    }

    annotation_text = @{
        (!"]" ~ ANY)*
    }

    time = {
       double_digits 
        ~ ":"
        ~ double_digits
    }

    double_digits = @{
        ASCII_DIGIT ~ ASCII_DIGIT
    }

    ident = @{
        ASCII_ALPHANUMERIC+
    }
"#]
struct Grammar;

impl FromStr for Schedule {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut schedule = Schedule::new();

        if s.trim().is_empty() {
            return Ok(schedule);
        }

        fn assert_rule(expected: Rule, val: &Pair<'_, Rule>) -> Result<(), String> {
            if val.as_rule() != expected {
                return Err(format!(
                    "expected {expected:?}, got {:?} for {:?}",
                    val.as_rule(),
                    val.as_str()
                ));
            }

            Ok(())
        }

        fn convert_time(x: Pair<'_, Rule>) -> Result<ExtendedTime, Box<dyn std::error::Error>> {
            assert_rule(Rule::time, &x)?;
            let mut time_iter = x.into_inner();

            ExtendedTime::new(
                time_iter.next().ok_or("missing hours")?.as_str().parse()?,
                time_iter
                    .next()
                    .ok_or("missing minutes")?
                    .as_str()
                    .parse()?,
            )
            .ok_or("invalid extend time".into())
        }

        fn convert_rule(
            x: Pair<'_, Rule>,
        ) -> Result<(RuleKind, Option<&'_ str>), Box<dyn std::error::Error>> {
            assert_rule(Rule::rule, &x)?;
            let mut rule_iter = x.into_inner();
            let kind = rule_iter.next().ok_or("missing kind")?.as_str().parse()?;

            let annotation = rule_iter.next().map(|inner| {
                let len = inner.as_str().len();
                &inner.as_str()[1..len - 1]
            });

            Ok((kind, annotation))
        }

        let root = Grammar::parse(Rule::schedule, s)?
            .next()
            .ok_or("missing schedule")?;

        for chain in root.into_inner() {
            if chain.as_rule() == Rule::EOI {
                break;
            }

            assert_rule(Rule::chain, &chain)?;
            let mut chain_iter = chain.into_inner();
            let mut curr_start = convert_time(chain_iter.next().ok_or("missing start time")?)?;

            while let Some(rule) = chain_iter.next() {
                let (kind, annotation) = convert_rule(rule)?;
                let curr_end = convert_time(chain_iter.next().ok_or("missing start time")?)?;

                schedule = schedule.addition(Schedule::from_ranges(
                    [curr_start..curr_end],
                    kind,
                    annotation.unwrap_or_default().into(),
                ));

                curr_start = curr_end
            }
        }

        Ok(schedule)
    }
}
