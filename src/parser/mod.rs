use std::cmp::Ord;
use std::convert::TryInto;
use std::fmt::Debug;
use std::hash::Hash;

use pest::iterators::Pair;
use pest::Parser;

use crate::time_domain as td;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct CSVParser;

#[derive(Clone, Debug)]
pub struct Error {
    pub description: String,
}

pub enum PlusOrMinus {
    Plus,
    Minus,
}

impl<T> From<pest::error::Error<T>> for Error
where
    T: Copy + Debug + Ord + Hash,
{
    fn from(pest_err: pest::error::Error<T>) -> Error {
        Error {
            description: format!("{}", pest_err),
        }
    }
}

pub fn unexpected_token<T>(token: Rule, parent: Rule) -> T {
    panic!(
        "grammar error: found `{:?}` inside of `{:?}`",
        token, parent
    )
}

pub fn parse(data: &str) -> Result<td::TimeDomain, Error> {
    let time_domain_pair = CSVParser::parse(Rule::time_domain, data)
        .map_err(Error::from)?
        .next()
        .expect("grammar error: no time_domain found");

    Ok(build_time_domain(time_domain_pair))
}

pub fn build_time_domain(pair: Pair<Rule>) -> td::TimeDomain {
    assert_eq!(pair.as_rule(), Rule::time_domain);

    let rules = pair
        .into_inner()
        .map(|pair| match pair.as_rule() {
            Rule::rule_sequence => build_rule_sequence(pair),
            t => unexpected_token(t, Rule::time_domain),
        })
        .collect();

    td::TimeDomain { rules }
}

pub fn build_rule_sequence(pair: Pair<Rule>) -> td::RuleSequence {
    assert_eq!(pair.as_rule(), Rule::rule_sequence);
    let mut pairs = pair.into_inner();
    println!("----------------------\n{:#?}", &pairs);

    let selector_sequence_pair = pairs.next().expect("grammar error: empty rule sequence");
    let rules_modifier_pair = pairs.next();

    let selector = build_selector_sequence(selector_sequence_pair);
    let (modifier, comment) = rules_modifier_pair
        .map(build_rules_modifier)
        .unwrap_or((td::RulesModifier::Open, None));

    td::RuleSequence {
        modifier,
        comment,
        selector,
    }
}

pub fn build_rules_modifier(pair: Pair<Rule>) -> (td::RulesModifier, Option<String>) {
    assert_eq!(pair.as_rule(), Rule::rules_modifier);
    let mut pairs = pair.into_inner();

    let (modifier_pair, comment_pair) = match (pairs.next(), pairs.next()) {
        (Some(modifier_pair), Some(comment_pair)) => {
            assert_eq!(modifier_pair.as_rule(), Rule::rules_modifier_enum);
            assert_eq!(comment_pair.as_rule(), Rule::comment);
            (Some(modifier_pair), Some(comment_pair))
        }
        (Some(pair), None) if pair.as_rule() == Rule::rules_modifier_enum => (Some(pair), None),
        (Some(pair), None) if pair.as_rule() == Rule::comment => (None, Some(pair)),
        _ => todo!(),
    };

    let comment = comment_pair.map(|pair| pair.as_str().to_string());
    let modifier = modifier_pair
        .map(build_rules_modifier_enum)
        .unwrap_or(td::RulesModifier::Open);

    (modifier, comment)
}

pub fn build_rules_modifier_enum(pair: Pair<Rule>) -> td::RulesModifier {
    assert_eq!(pair.as_rule(), Rule::rules_modifier_enum);

    let pair = pair
        .into_inner()
        .next()
        .expect("grammar error: empty rules modifier enum");

    match pair.as_rule() {
        Rule::rules_modifier_enum_closed => td::RulesModifier::Closed,
        Rule::rules_modifier_enum_open => td::RulesModifier::Open,
        Rule::rules_modifier_enum_unknown => td::RulesModifier::Unknown,
        other => unexpected_token(other, Rule::rules_modifier_enum),
    }
}

pub fn build_selector_sequence(pair: Pair<Rule>) -> td::Selector {
    assert_eq!(pair.as_rule(), Rule::selector_sequence);

    let mut time_selector = td::Selector::always_open();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::always_open => {}
            Rule::wide_range_selectors => {
                let (year_selector, monthday_selector, week_selector) =
                    build_wide_range_selectors(pair);

                if let Some(year_selector) = year_selector {
                    time_selector.year = year_selector;
                }

                if let Some(monthday_selector) = monthday_selector {
                    time_selector.monthday = monthday_selector;
                }

                if let Some(week_selector) = week_selector {
                    time_selector.week = week_selector;
                }
            }
            Rule::small_range_selectors => {}
            other => unexpected_token(other, Rule::selector_sequence),
        }
    }

    time_selector
}

pub fn build_wide_range_selectors(
    pair: Pair<Rule>,
) -> (
    Option<td::YearSelector>,
    Option<td::MonthdaySelector>,
    Option<td::WeekSelector>,
) {
    assert_eq!(pair.as_rule(), Rule::wide_range_selectors);

    let mut year_selector = None;
    let mut monthday_selector = None;
    let mut week_selector = None;

    let pairs = pair.into_inner();
    println!("{:#?}", pairs);
    println!("-----------");

    for pair in pairs {
        match pair.as_rule() {
            Rule::year_selector => year_selector = Some(build_year_selector(pair)),
            Rule::monthday_selector => monthday_selector = Some(build_monthday_selector(pair)),
            Rule::week_selector => week_selector = Some(build_week_selector(pair)),
            other => unexpected_token(other, Rule::wide_range_selectors),
        }
    }

    (year_selector, monthday_selector, week_selector)
}

pub fn build_year_selector(pair: Pair<Rule>) -> td::YearSelector {
    assert_eq!(pair.as_rule(), Rule::year_selector);
    td::YearSelector::new(pair.into_inner().map(build_year_range))
}

pub fn build_year_range(pair: Pair<Rule>) -> td::YearRange {
    assert_eq!(pair.as_rule(), Rule::year_range);
    let mut rules = pair.into_inner();

    let start = build_year(rules.next().expect("empty year range"));
    let end = rules.next().map(|pair| match pair.as_rule() {
        Rule::year => build_year(pair),
        Rule::year_range_plus => 9999,
        other => unexpected_token(other, Rule::year_range),
    });
    let step = rules.next().map(build_year);

    td::YearRange {
        start,
        end: end.unwrap_or(start),
        step: step.unwrap_or(1),
    }
}

pub fn build_year(pair: Pair<Rule>) -> u16 {
    assert_eq!(pair.as_rule(), Rule::year);
    pair.as_str().parse().expect("invalid year format")
}

pub fn build_monthday_selector(pair: Pair<Rule>) -> td::MonthdaySelector {
    assert_eq!(pair.as_rule(), Rule::monthday_selector);
    td::MonthdaySelector::new(pair.into_inner().into_iter().map(build_monthday_range))
}

pub fn build_monthday_range(pair: Pair<Rule>) -> td::MonthdayRange {
    assert_eq!(pair.as_rule(), Rule::monthday_range);
    let mut pairs = pair.into_inner().into_iter();

    let year = {
        if pairs.peek().map(|x| x.as_rule()) == Some(Rule::year) {
            Some(build_year(pairs.next().unwrap()))
        } else {
            None
        }
    };

    match pairs.peek().expect("empty monthday range").as_rule() {
        Rule::month => {
            let start = build_month(pairs.next().unwrap());
            let end = pairs.next().map(build_month).unwrap_or(start);
            td::MonthdayRange::Month { year, start, end }
        }
        Rule::date_from => {
            let start = build_date_from(pairs.next().unwrap());

            let start_offset = {
                if pairs.peek().map(|x| x.as_rule()) == Some(Rule::date_offset) {
                    build_date_offset(pairs.next().unwrap())
                } else {
                    td::DateOffset::default()
                }
            };

            let end = match pairs.peek().map(|x| x.as_rule()) {
                Some(Rule::date_to) => build_date_to(pairs.next().unwrap()),
                Some(Rule::monthday_range_plus) => td::DateTo::day(31, td::Month::December, 9999),
                None => td::DateTo::DateFrom(start.clone()),
                Some(other) => unexpected_token(other, Rule::monthday_range),
            };

            let end_offset = pairs.next().map(build_date_offset).unwrap_or_default();

            td::MonthdayRange::Date {
                start: (start, start_offset),
                end: (end, end_offset),
            }
        }
        other => unexpected_token(other, Rule::monthday_range),
    }
}

pub fn build_date_from(pair: Pair<Rule>) -> td::DateFrom {
    assert_eq!(pair.as_rule(), Rule::date_from);
    let mut pairs = pair.into_inner().into_iter();

    let year = {
        if pairs.peek().map(|x| x.as_rule()) == Some(Rule::year) {
            Some(build_year(pairs.next().unwrap()))
        } else {
            None
        }
    };

    match pairs.peek().expect("empty date (from)").as_rule() {
        Rule::variable_date => td::DateFrom::Easter { year },
        Rule::month => td::DateFrom::Fixed {
            year,
            month: build_month(pairs.next().expect("missing month")),
            day: build_daynum(pairs.next().expect("missing day")),
        },
        other => unexpected_token(other, Rule::date_from),
    }
}

pub fn build_date_to(pair: Pair<Rule>) -> td::DateTo {
    assert_eq!(pair.as_rule(), Rule::date_to);
    let pair = pair.into_inner().next().expect("empty date (to)");

    match pair.as_rule() {
        Rule::date_from => td::DateTo::DateFrom(build_date_from(pair)),
        Rule::daynum => td::DateTo::DayNum(build_daynum(pair)),
        other => unexpected_token(other, Rule::date_to),
    }
}

pub fn build_date_offset(pair: Pair<Rule>) -> td::DateOffset {
    assert_eq!(pair.as_rule(), Rule::date_offset);
    let mut pairs = pair.into_inner().into_iter();

    let wday_offset = {
        if pairs.peek().map(|x| x.as_rule()) == Some(Rule::plus_or_minus) {
            let sign = build_plus_or_minus(pairs.next().unwrap());
            let wday = build_wday(pairs.next().expect("missing wday after sign"));

            match sign {
                PlusOrMinus::Plus => td::WeekDayOffset::Next(wday),
                PlusOrMinus::Minus => td::WeekDayOffset::Prev(wday),
            }
        } else {
            td::WeekDayOffset::None
        }
    };

    let day_offset = pairs.next().map(build_day_offset).unwrap_or(0);

    td::DateOffset {
        wday_offset,
        day_offset,
    }
}

pub fn build_plus_or_minus(pair: Pair<Rule>) -> PlusOrMinus {
    assert_eq!(pair.as_rule(), Rule::plus_or_minus);
    let pair = pair.into_inner().next().expect("empty plus or minus");

    match pair.as_rule() {
        Rule::plus => PlusOrMinus::Plus,
        Rule::minus => PlusOrMinus::Minus,
        other => unexpected_token(other, Rule::plus_or_minus),
    }
}

pub fn build_day_offset(pair: Pair<Rule>) -> i64 {
    assert_eq!(pair.as_rule(), Rule::day_offset);
    let mut pairs = pair.into_inner().into_iter();

    let sign = build_plus_or_minus(pairs.next().expect("empty day offset"));
    let val_abs: i64 = build_positive_number(pairs.next().expect("missing value"))
        .try_into()
        .expect("day offset value too large"); // TODO: this should probably be raised in a result

    match sign {
        PlusOrMinus::Plus => val_abs,
        PlusOrMinus::Minus => -val_abs,
    }
}

pub fn build_positive_number(pair: Pair<Rule>) -> u64 {
    assert_eq!(pair.as_rule(), Rule::positive_number);
    pair.as_str().parse().expect("invalid positive_number")
}

pub fn build_wday(pair: Pair<Rule>) -> td::Weekday {
    assert_eq!(pair.as_rule(), Rule::wday);
    let pair = pair.into_inner().next().expect("empty week day");

    match pair.as_rule() {
        Rule::sunday => td::Weekday::Sunday,
        Rule::monday => td::Weekday::Monday,
        Rule::tuesday => td::Weekday::Tuesday,
        Rule::wednesday => td::Weekday::Wednesday,
        Rule::thursday => td::Weekday::Thursday,
        Rule::friday => td::Weekday::Friday,
        Rule::saturday => td::Weekday::Saturday,
        other => unexpected_token(other, Rule::wday),
    }
}

pub fn build_month(pair: Pair<Rule>) -> td::Month {
    assert_eq!(pair.as_rule(), Rule::month);
    let pair = pair.into_inner().next().expect("empty month");

    match pair.as_rule() {
        Rule::january => td::Month::January,
        Rule::february => td::Month::February,
        Rule::march => td::Month::March,
        Rule::april => td::Month::April,
        Rule::may => td::Month::May,
        Rule::june => td::Month::June,
        Rule::july => td::Month::July,
        Rule::august => td::Month::August,
        Rule::september => td::Month::September,
        Rule::october => td::Month::October,
        Rule::november => td::Month::November,
        Rule::december => td::Month::December,
        other => unexpected_token(other, Rule::month),
    }
}

pub fn build_daynum(pair: Pair<Rule>) -> u8 {
    assert_eq!(pair.as_rule(), Rule::daynum);
    pair.as_str().parse().expect("invalid month format")
}

pub fn build_week_selector(pair: Pair<Rule>) -> td::WeekSelector {
    assert_eq!(pair.as_rule(), Rule::week_selector);
    td::WeekSelector::new(pair.into_inner().into_iter().map(build_week))
}

pub fn build_week(pair: Pair<Rule>) -> td::WeekRange {
    assert_eq!(pair.as_rule(), Rule::week);
    let mut rules = pair.into_inner();

    let start = build_weeknum(rules.next().expect("empty weeknum range"));
    let end = rules.next().map(build_weeknum);
    let step = rules.next().map(build_weeknum);

    td::WeekRange {
        start,
        end: end.unwrap_or(start),
        step: step.unwrap_or(1),
    }
}

pub fn build_weeknum(pair: Pair<Rule>) -> u8 {
    assert_eq!(pair.as_rule(), Rule::weeknum);
    pair.as_str().parse().expect("invalid weeknum format")
}
