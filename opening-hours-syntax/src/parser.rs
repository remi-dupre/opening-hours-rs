use std::cmp::Ord;
use std::convert::TryInto;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::RangeInclusive;
use std::sync::Arc;

use chrono::Duration;

use pest::iterators::Pair;
use pest::Parser;

use crate::error::{Error, Result};
use crate::extended_time::ExtendedTime;
use crate::rules as rl;
use crate::rules::day as ds;
use crate::rules::time as ts;

#[cfg(feature = "log")]
static WARN_EASTER: std::sync::Once = std::sync::Once::new();

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct OHParser;

/// Just used while collecting parsed expression
enum Sign {
    Neg,
    Pos,
}

pub fn parse(data: &str) -> Result<rl::OpeningHoursExpression> {
    let opening_hours_pair = OHParser::parse(Rule::input_opening_hours, data)
        .map_err(Error::from)?
        .next()
        .expect("grammar error: no opening_hours found");

    let rules = build_opening_hours(opening_hours_pair)?;
    Ok(rl::OpeningHoursExpression { rules })
}

// ---
// --- Time domain
// ---

fn unexpected_token<T>(token: Rule, parent: Rule) -> T {
    unreachable!("Grammar error: found `{token:?}` inside of `{parent:?}`")
}

fn build_opening_hours(pair: Pair<Rule>) -> Result<Vec<rl::RuleSequence>> {
    assert_eq!(pair.as_rule(), Rule::opening_hours);
    let mut pairs = pair.into_inner();
    let mut rules = Vec::new();

    while let Some(pair) = pairs.next() {
        rules.push(match pair.as_rule() {
            Rule::rule_sequence => build_rule_sequence(pair, rl::RuleOperator::Normal),
            Rule::any_rule_separator => build_rule_sequence(
                pairs.next().expect("separator not followed by any rule"),
                build_any_rule_separator(pair),
            ),
            other => unexpected_token(other, Rule::opening_hours),
        }?)
    }

    Ok(rules)
}

fn build_rule_sequence(pair: Pair<Rule>, operator: rl::RuleOperator) -> Result<rl::RuleSequence> {
    assert_eq!(pair.as_rule(), Rule::rule_sequence);
    let mut pairs = pair.into_inner();

    let (day_selector, time_selector, extra_comment) =
        build_selector_sequence(pairs.next().expect("grammar error: empty rule sequence"))?;

    let (kind, comment) = pairs
        .next()
        .map(build_rules_modifier)
        .unwrap_or((rl::RuleKind::Open, None));

    let comments = comment
        .into_iter()
        .chain(extra_comment)
        .map(|s| Arc::from(s.into_boxed_str()))
        .collect::<Vec<_>>()
        .into();

    Ok(rl::RuleSequence {
        day_selector,
        time_selector,
        kind,
        operator,
        comments,
    })
}

fn build_any_rule_separator(pair: Pair<Rule>) -> rl::RuleOperator {
    assert_eq!(pair.as_rule(), Rule::any_rule_separator);

    match pair
        .into_inner()
        .next()
        .expect("empty rule separator")
        .as_rule()
    {
        Rule::normal_rule_separator => rl::RuleOperator::Normal,
        Rule::additional_rule_separator => rl::RuleOperator::Additional,
        Rule::fallback_rule_separator => rl::RuleOperator::Fallback,
        other => unexpected_token(other, Rule::any_rule_separator),
    }
}

// ---
// --- Rule modifier
// ---

fn build_rules_modifier(pair: Pair<Rule>) -> (rl::RuleKind, Option<String>) {
    assert_eq!(pair.as_rule(), Rule::rules_modifier);
    let mut pairs = pair.into_inner();

    let kind = {
        if pairs.peek().expect("empty rules_modifier").as_rule() == Rule::rules_modifier_enum {
            build_rules_modifier_enum(pairs.next().unwrap())
        } else {
            rl::RuleKind::Open
        }
    };

    let comment = pairs.next().map(build_comment);
    (kind, comment)
}

fn build_rules_modifier_enum(pair: Pair<Rule>) -> rl::RuleKind {
    assert_eq!(pair.as_rule(), Rule::rules_modifier_enum);

    let pair = pair
        .into_inner()
        .next()
        .expect("grammar error: empty rules modifier enum");

    match pair.as_rule() {
        Rule::rules_modifier_enum_closed => rl::RuleKind::Closed,
        Rule::rules_modifier_enum_open => rl::RuleKind::Open,
        Rule::rules_modifier_enum_unknown => rl::RuleKind::Unknown,
        other => unexpected_token(other, Rule::rules_modifier_enum),
    }
}

// ---
// --- Selectors
// ---

fn build_selector_sequence(
    pair: Pair<Rule>,
) -> Result<(ds::DaySelector, ts::TimeSelector, Option<String>)> {
    assert_eq!(pair.as_rule(), Rule::selector_sequence);
    let mut pairs = pair.into_inner();

    if pairs.peek().map(|x| x.as_rule()).expect("empty selector") == Rule::always_open {
        return Ok(Default::default());
    }

    let (year, monthday, week, comment) = {
        if pairs.peek().map(|x| x.as_rule()).unwrap() == Rule::wide_range_selectors {
            build_wide_range_selectors(pairs.next().unwrap())?
        } else {
            (Vec::new(), Vec::new(), Vec::new(), None)
        }
    };

    let (weekday, time) = {
        if let Some(pair) = pairs.next() {
            build_small_range_selectors(pair)?
        } else {
            (Vec::new(), Vec::new())
        }
    };

    Ok((
        ds::DaySelector { year, monthday, week, weekday },
        ts::TimeSelector::new(time),
        comment,
    ))
}

#[allow(clippy::type_complexity)]
fn build_wide_range_selectors(
    pair: Pair<Rule>,
) -> Result<(
    Vec<ds::YearRange>,
    Vec<ds::MonthdayRange>,
    Vec<ds::WeekRange>,
    Option<String>,
)> {
    assert_eq!(pair.as_rule(), Rule::wide_range_selectors);

    let mut year_selector = Vec::new();
    let mut monthday_selector = Vec::new();
    let mut week_selector = Vec::new();
    let mut comment = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::year_selector => year_selector = build_year_selector(pair)?,
            Rule::monthday_selector => monthday_selector = build_monthday_selector(pair)?,
            Rule::week_selector => week_selector = build_week_selector(pair)?,
            Rule::comment => comment = Some(build_comment(pair)),
            other => unexpected_token(other, Rule::wide_range_selectors),
        }
    }

    Ok((year_selector, monthday_selector, week_selector, comment))
}

fn build_small_range_selectors(
    pair: Pair<Rule>,
) -> Result<(Vec<ds::WeekDayRange>, Vec<ts::TimeSpan>)> {
    assert_eq!(pair.as_rule(), Rule::small_range_selectors);

    let mut weekday_selector = Vec::new();
    let mut time_selector = Vec::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::weekday_selector => weekday_selector = build_weekday_selector(pair)?,
            Rule::time_selector => time_selector = build_time_selector(pair)?,
            other => unexpected_token(other, Rule::wide_range_selectors),
        }
    }

    Ok((weekday_selector, time_selector))
}

// ---
// --- Time selector
// ---

fn build_time_selector(pair: Pair<Rule>) -> Result<Vec<ts::TimeSpan>> {
    assert_eq!(pair.as_rule(), Rule::time_selector);
    pair.into_inner().map(build_timespan).collect()
}

fn build_timespan(pair: Pair<Rule>) -> Result<ts::TimeSpan> {
    assert_eq!(pair.as_rule(), Rule::timespan);
    let mut pairs = pair.into_inner();

    let start = build_time(pairs.next().expect("empty timespan"))?;

    let end = match pairs.next() {
        None => {
            // TODO: opening_hours.js handles this better: it will set the
            //       state to unknown and add a warning comment.
            ts::Time::Fixed(ExtendedTime::new(24, 0).unwrap())
        }
        Some(pair) if pair.as_rule() == Rule::timespan_plus => {
            return Err(Error::Unsupported("point in time"))
        }
        Some(pair) => build_extended_time(pair)?,
    };

    let (open_end, repeats) = match pairs.peek().map(|x| x.as_rule()) {
        None => (false, None),
        Some(Rule::timespan_plus) => (true, None),
        Some(Rule::minute) => (false, Some(build_minute(pairs.next().unwrap()))),
        Some(Rule::hour_minutes) => (
            false,
            Some(build_hour_minutes_as_duration(pairs.next().unwrap())),
        ),
        Some(other) => unexpected_token(other, Rule::timespan),
    };

    Ok(ts::TimeSpan { range: start..end, repeats, open_end })
}

fn build_time(pair: Pair<Rule>) -> Result<ts::Time> {
    assert_eq!(pair.as_rule(), Rule::time);
    let inner = pair.into_inner().next().expect("empty time");

    Ok(match inner.as_rule() {
        Rule::hour_minutes => ts::Time::Fixed(build_hour_minutes(inner)?),
        Rule::variable_time => ts::Time::Variable(build_variable_time(inner)?),
        other => unexpected_token(other, Rule::time),
    })
}

fn build_extended_time(pair: Pair<Rule>) -> Result<ts::Time> {
    assert_eq!(pair.as_rule(), Rule::extended_time);
    let inner = pair.into_inner().next().expect("empty extended time");

    Ok(match inner.as_rule() {
        Rule::extended_hour_minutes => ts::Time::Fixed(build_extended_hour_minutes(inner)?),
        Rule::variable_time => ts::Time::Variable(build_variable_time(inner)?),
        other => unexpected_token(other, Rule::extended_time),
    })
}

fn build_variable_time(pair: Pair<Rule>) -> Result<ts::VariableTime> {
    assert_eq!(pair.as_rule(), Rule::variable_time);
    let mut pairs = pair.into_inner();

    let event = build_event(pairs.next().expect("empty variable time"));

    let offset = {
        if pairs.peek().is_some() {
            let sign = build_plus_or_minus(pairs.next().unwrap());

            let mins: i16 = build_hour_minutes(pairs.next().expect("missing hour minutes"))?
                .mins_from_midnight()
                .try_into()
                .expect("offset overflow");

            match sign {
                PlusOrMinus::Plus => mins,
                PlusOrMinus::Minus => -mins,
            }
        } else {
            0
        }
    };

    Ok(ts::VariableTime { event, offset })
}

fn build_event(pair: Pair<Rule>) -> ts::TimeEvent {
    assert_eq!(pair.as_rule(), Rule::event);

    match pair.into_inner().next().expect("empty event").as_rule() {
        Rule::dawn => ts::TimeEvent::Dawn,
        Rule::sunrise => ts::TimeEvent::Sunrise,
        Rule::sunset => ts::TimeEvent::Sunset,
        Rule::dusk => ts::TimeEvent::Dusk,
        other => unexpected_token(other, Rule::event),
    }
}

// ---
// --- WeekDay selector
// ---

fn build_weekday_selector(pair: Pair<Rule>) -> Result<Vec<ds::WeekDayRange>> {
    assert_eq!(pair.as_rule(), Rule::weekday_selector);

    pair.into_inner()
        .flat_map(|pair| match pair.as_rule() {
            Rule::weekday_sequence => pair.into_inner().map(build_weekday_range as fn(_) -> _),
            Rule::holiday_sequence => pair.into_inner().map(build_holiday as _),
            other => unexpected_token(other, Rule::weekday_sequence),
        })
        .collect()
}

fn build_weekday_range(pair: Pair<Rule>) -> Result<ds::WeekDayRange> {
    assert_eq!(pair.as_rule(), Rule::weekday_range);
    let mut pairs = pair.into_inner();

    let start = build_wday(pairs.next().expect("empty weekday range"));

    let end = {
        if pairs.peek().map(|x| x.as_rule()) == Some(Rule::wday) {
            build_wday(pairs.next().unwrap())
        } else {
            start
        }
    };

    let mut nth_from_start = [false; 5];
    let mut nth_from_end = [false; 5];

    while pairs.peek().map(|x| x.as_rule()) == Some(Rule::nth_entry) {
        let (sign, indices) = build_nth_entry(pairs.next().unwrap())?;

        let nth_array = match sign {
            Sign::Neg => &mut nth_from_end,
            Sign::Pos => &mut nth_from_start,
        };

        for i in indices {
            nth_array[usize::from(i - 1)] = true;
        }
    }

    if !nth_from_start.contains(&true) && !nth_from_end.contains(&true) {
        nth_from_start = [true; 5];
        nth_from_end = [true; 5];
    }

    let offset = {
        if let Some(pair) = pairs.next() {
            build_day_offset(pair)?
        } else {
            0
        }
    };

    Ok(ds::WeekDayRange::Fixed {
        range: start..=end,
        offset,
        nth_from_start,
        nth_from_end,
    })
}

fn build_holiday(pair: Pair<Rule>) -> Result<ds::WeekDayRange> {
    assert_eq!(pair.as_rule(), Rule::holiday);
    let mut pairs = pair.into_inner();

    let kind = match pairs.next().expect("empty holiday").as_rule() {
        Rule::public_holiday => ds::HolidayKind::Public,
        Rule::school_holiday => ds::HolidayKind::School,
        other => unexpected_token(other, Rule::holiday),
    };

    let offset = pairs.next().map(build_day_offset).unwrap_or(Ok(0))?;
    Ok(ds::WeekDayRange::Holiday { kind, offset })
}

fn build_nth_entry(pair: Pair<Rule>) -> Result<(Sign, RangeInclusive<u8>)> {
    assert_eq!(pair.as_rule(), Rule::nth_entry);
    let mut pairs = pair.into_inner();

    let sign = {
        if pairs.peek().map(|x| x.as_rule()) == Some(Rule::nth_minus) {
            pairs.next();
            Sign::Neg
        } else {
            Sign::Pos
        }
    };

    let start = build_nth(pairs.next().expect("empty nth entry"));
    let end = pairs.next().map(build_nth).unwrap_or(start);
    Ok((sign, start..=end))
}

fn build_nth(pair: Pair<Rule>) -> u8 {
    assert_eq!(pair.as_rule(), Rule::nth);
    pair.as_str().parse().expect("invalid nth format")
}

fn build_day_offset(pair: Pair<Rule>) -> Result<i64> {
    assert_eq!(pair.as_rule(), Rule::day_offset);
    let mut pairs = pair.into_inner();

    let sign = build_plus_or_minus(pairs.next().expect("empty day offset"));
    let val_abs = build_positive_number(pairs.next().expect("missing value"))?;

    let val_abs: i64 = val_abs.try_into().map_err(|_| Error::Overflow {
        value: format!("{}", val_abs),
        expected: "an integer in [-2**63, 2**63[".to_string(),
    })?;

    Ok(match sign {
        PlusOrMinus::Plus => val_abs,
        PlusOrMinus::Minus => -val_abs,
    })
}

// ---
// --- Week selector
// ---

fn build_week_selector(pair: Pair<Rule>) -> Result<Vec<ds::WeekRange>> {
    assert_eq!(pair.as_rule(), Rule::week_selector);
    pair.into_inner().map(build_week).collect()
}

fn build_week(pair: Pair<Rule>) -> Result<ds::WeekRange> {
    assert_eq!(pair.as_rule(), Rule::week);
    let mut rules = pair.into_inner();

    let start = build_weeknum(rules.next().expect("empty weeknum range"));
    let end = rules.next().map(build_weeknum);

    let step = rules.next().map(build_positive_number).transpose()?;
    let step = step.unwrap_or(1).try_into().map_err(|_| Error::Overflow {
        value: format!("{}", step.unwrap()),
        expected: "an integer in [0, 255]".to_string(),
    })?;

    Ok(ds::WeekRange { range: start..=end.unwrap_or(start), step })
}

// ---
// --- Month selector
// ---

fn build_monthday_selector(pair: Pair<Rule>) -> Result<Vec<ds::MonthdayRange>> {
    assert_eq!(pair.as_rule(), Rule::monthday_selector);
    pair.into_inner().map(build_monthday_range).collect()
}

fn build_monthday_range(pair: Pair<Rule>) -> Result<ds::MonthdayRange> {
    assert_eq!(pair.as_rule(), Rule::monthday_range);
    let mut pairs = pair.into_inner();

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

            Ok(ds::MonthdayRange::Month { year, range: start..=end })
        }
        Rule::date_from => {
            let start = build_date_from(pairs.next().unwrap());

            let start_offset = {
                if pairs.peek().map(|x| x.as_rule()) == Some(Rule::date_offset) {
                    build_date_offset(pairs.next().unwrap())?
                } else {
                    ds::DateOffset::default()
                }
            };

            let end = match pairs.peek().map(|x| x.as_rule()) {
                Some(Rule::date_to) => build_date_to(pairs.next().unwrap(), start)?,
                Some(Rule::monthday_range_plus) => {
                    pairs.next();

                    if start.has_year() {
                        ds::Date::ymd(31, ds::Month::December, 9999)
                    } else {
                        ds::Date::md(31, ds::Month::December)
                    }
                }
                None => start,
                Some(other) => unexpected_token(other, Rule::monthday_range),
            };

            let end_offset = pairs
                .next()
                .map(build_date_offset)
                .unwrap_or_else(|| Ok(Default::default()))?;

            Ok(ds::MonthdayRange::Date {
                start: (start, start_offset),
                end: (end, end_offset),
            })
        }
        other => unexpected_token(other, Rule::monthday_range),
    }
}

fn build_date_offset(pair: Pair<Rule>) -> Result<ds::DateOffset> {
    assert_eq!(pair.as_rule(), Rule::date_offset);
    let mut pairs = pair.into_inner();

    let wday_offset = {
        if pairs.peek().map(|x| x.as_rule()) == Some(Rule::plus_or_minus) {
            let sign = build_plus_or_minus(pairs.next().unwrap());
            let wday = build_wday(pairs.next().expect("missing wday after sign"));

            match sign {
                PlusOrMinus::Plus => ds::WeekDayOffset::Next(wday),
                PlusOrMinus::Minus => ds::WeekDayOffset::Prev(wday),
            }
        } else {
            ds::WeekDayOffset::None
        }
    };

    let day_offset = pairs.next().map(build_day_offset).unwrap_or(Ok(0))?;

    Ok(ds::DateOffset { wday_offset, day_offset })
}

fn build_date_from(pair: Pair<Rule>) -> ds::Date {
    assert_eq!(pair.as_rule(), Rule::date_from);
    let mut pairs = pair.into_inner();

    let year = {
        if pairs.peek().map(|x| x.as_rule()) == Some(Rule::year) {
            Some(build_year(pairs.next().unwrap()))
        } else {
            None
        }
    };

    match pairs.peek().expect("empty date (from)").as_rule() {
        Rule::variable_date => {
            #[cfg(feature = "log")]
            WARN_EASTER.call_once(|| log::warn!("Easter is not supported yet"));
            ds::Date::Easter { year }
        }
        Rule::month => ds::Date::Fixed {
            year,
            month: build_month(pairs.next().expect("missing month")),
            day: build_daynum(pairs.next().expect("missing day")),
        },
        other => unexpected_token(other, Rule::date_from),
    }
}

fn build_date_to(pair: Pair<Rule>, from: ds::Date) -> Result<ds::Date> {
    assert_eq!(pair.as_rule(), Rule::date_to);
    let pair = pair.into_inner().next().expect("empty date (to)");

    Ok(match pair.as_rule() {
        Rule::date_from => build_date_from(pair),
        Rule::daynum => {
            let daynum = build_daynum(pair);

            match from {
                ds::Date::Easter { .. } => {
                    // NOTE: this is actually not a specified constraint, but it is quite confusing
                    //       that this is allowed
                    return Err(Error::Unsupported("Easter followed by a day number"));
                }
                ds::Date::Fixed { mut year, mut month, day } => {
                    if day > daynum {
                        month = month.next();

                        if month == ds::Month::January {
                            if let Some(x) = year.as_mut() {
                                *x += 1
                            }
                        }
                    }

                    ds::Date::Fixed { year, month, day: daynum }
                }
            }
        }
        other => unexpected_token(other, Rule::date_to),
    })
}

// ---
// --- Year selector
// ---

fn build_year_selector(pair: Pair<Rule>) -> Result<Vec<ds::YearRange>> {
    assert_eq!(pair.as_rule(), Rule::year_selector);
    pair.into_inner().map(build_year_range).collect()
}

fn build_year_range(pair: Pair<Rule>) -> Result<ds::YearRange> {
    assert_eq!(pair.as_rule(), Rule::year_range);
    let mut rules = pair.into_inner();

    let start = build_year(rules.next().expect("empty year range"));
    let end = rules.next().map(|pair| match pair.as_rule() {
        Rule::year => build_year(pair),
        Rule::year_range_plus => 9999,
        other => unexpected_token(other, Rule::year_range),
    });

    let step = rules.next().map(build_positive_number).transpose()?;
    let step = step.unwrap_or(1).try_into().map_err(|_| Error::Overflow {
        value: format!("{}", step.unwrap()),
        expected: "an integer in [0, 2**16[".to_string(),
    })?;

    Ok(ds::YearRange { range: start..=end.unwrap_or(start), step })
}

// ---
// --- Basic elements
// ---

fn build_plus_or_minus(pair: Pair<Rule>) -> PlusOrMinus {
    assert_eq!(pair.as_rule(), Rule::plus_or_minus);
    let pair = pair.into_inner().next().expect("empty plus or minus");

    match pair.as_rule() {
        Rule::plus => PlusOrMinus::Plus,
        Rule::minus => PlusOrMinus::Minus,
        other => unexpected_token(other, Rule::plus_or_minus),
    }
}

fn build_minute(pair: Pair<Rule>) -> Duration {
    assert_eq!(pair.as_rule(), Rule::minute);
    let minutes = pair.as_str().parse().expect("invalid minute");
    Duration::minutes(minutes)
}

fn build_hour_minutes(pair: Pair<Rule>) -> Result<ExtendedTime> {
    assert_eq!(pair.as_rule(), Rule::hour_minutes);
    let mut pairs = pair.into_inner();

    let hour = pairs
        .next()
        .expect("missing hour")
        .as_str()
        .parse()
        .expect("invalid hour");

    let minutes = pairs
        .next()
        .expect("missing minutes")
        .as_str()
        .parse()
        .expect("invalid minutes");

    ExtendedTime::new(hour, minutes).ok_or(Error::InvalidExtendTime { hour, minutes })
}

fn build_extended_hour_minutes(pair: Pair<Rule>) -> Result<ExtendedTime> {
    assert_eq!(pair.as_rule(), Rule::extended_hour_minutes);
    let mut pairs = pair.into_inner();

    let hour = pairs
        .next()
        .expect("missing hour")
        .as_str()
        .parse()
        .expect("invalid hour");

    let minutes = pairs
        .next()
        .expect("missing minutes")
        .as_str()
        .parse()
        .expect("invalid minutes");

    ExtendedTime::new(hour, minutes).ok_or(Error::InvalidExtendTime { hour, minutes })
}

fn build_hour_minutes_as_duration(pair: Pair<Rule>) -> Duration {
    assert_eq!(pair.as_rule(), Rule::hour_minutes);
    let mut pairs = pair.into_inner();

    let hour = pairs
        .next()
        .expect("missing hour")
        .as_str()
        .parse()
        .expect("invalid hour");

    let minutes = pairs
        .next()
        .expect("missing minutes")
        .as_str()
        .parse()
        .expect("invalid minutes");

    Duration::hours(hour) + Duration::minutes(minutes)
}

fn build_wday(pair: Pair<Rule>) -> ds::Weekday {
    assert_eq!(pair.as_rule(), Rule::wday);
    let pair = pair.into_inner().next().expect("empty week day");

    match pair.as_rule() {
        Rule::sunday => ds::Weekday::Sun,
        Rule::monday => ds::Weekday::Mon,
        Rule::tuesday => ds::Weekday::Tue,
        Rule::wednesday => ds::Weekday::Wed,
        Rule::thursday => ds::Weekday::Thu,
        Rule::friday => ds::Weekday::Fri,
        Rule::saturday => ds::Weekday::Sat,
        other => unexpected_token(other, Rule::wday),
    }
}

fn build_daynum(pair: Pair<Rule>) -> u8 {
    assert_eq!(pair.as_rule(), Rule::daynum);
    let daynum = pair.as_str().parse().expect("invalid month format");

    if daynum == 0 {
        #[cfg(feature = "log")]
        log::warn!("Found day number 0 in opening hours: specify the 1st or 31st instead.");
        return 1;
    }

    if daynum > 31 {
        #[cfg(feature = "log")]
        log::warn!("Found day number {daynum} in opening hours");
        return 31;
    }

    daynum
}

fn build_weeknum(pair: Pair<Rule>) -> u8 {
    assert_eq!(pair.as_rule(), Rule::weeknum);
    pair.as_str().parse().expect("invalid weeknum format")
}

fn build_month(pair: Pair<Rule>) -> ds::Month {
    assert_eq!(pair.as_rule(), Rule::month);
    let pair = pair.into_inner().next().expect("empty month");

    match pair.as_rule() {
        Rule::january => ds::Month::January,
        Rule::february => ds::Month::February,
        Rule::march => ds::Month::March,
        Rule::april => ds::Month::April,
        Rule::may => ds::Month::May,
        Rule::june => ds::Month::June,
        Rule::july => ds::Month::July,
        Rule::august => ds::Month::August,
        Rule::september => ds::Month::September,
        Rule::october => ds::Month::October,
        Rule::november => ds::Month::November,
        Rule::december => ds::Month::December,
        other => unexpected_token(other, Rule::month),
    }
}

fn build_year(pair: Pair<Rule>) -> u16 {
    assert_eq!(pair.as_rule(), Rule::year);
    pair.as_str().parse().expect("invalid year format")
}

fn build_positive_number(pair: Pair<Rule>) -> Result<u64> {
    assert_eq!(pair.as_rule(), Rule::positive_number);
    pair.as_str().parse().map_err(|_| Error::Overflow {
        value: pair.as_str().to_string(),
        expected: "a number between 0 and 2**64".to_string(),
    })
}

fn build_comment(pair: Pair<Rule>) -> String {
    assert_eq!(pair.as_rule(), Rule::comment);
    build_comment_inner(pair.into_inner().next().expect("empty comment"))
}

fn build_comment_inner(pair: Pair<Rule>) -> String {
    assert_eq!(pair.as_rule(), Rule::comment_inner);
    pair.as_str().to_string()
}

// Mics

enum PlusOrMinus {
    Plus,
    Minus,
}
