use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ord;
use core::convert::TryInto;
use core::fmt::Debug;
use core::hash::Hash;
use core::ops::RangeInclusive;

use chrono::Duration;

use pest::iterators::Pair;
use pest::Parser;

use crate::error::{Error, Result};
use crate::extended_time::ExtendedTime;
use crate::rules as rl;
use crate::rules::day::{self as ds, WeekNum, Year};
use crate::rules::time as ts;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct OHParser;

pub fn parse(data: &str) -> Result<rl::OpeningHoursExpression> {
    OHParser::parse(Rule::input_opening_hours, data)
        .map_err(Error::from)?
        .next()
        .ok_or(Error::GrammarLogic {
            rule: Rule::input_opening_hours,
            invariant: "cannot be missing",
        })
        .and_then(build_opening_hours)
        .map(|rules| rl::OpeningHoursExpression { rules })
        .inspect_err(|err| {
            debug_assert!(
                !err.is_implementation_error(),
                "parser implementation error: {err:?}",
            )
        })
}

impl alloc::str::FromStr for rl::OpeningHoursExpression {
    type Err = Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        parse(s)
    }
}

// --
// -- Time domain
// --

fn build_opening_hours(pair: Pair<Rule>) -> Result<Vec<rl::RuleSequence>> {
    debug_assert_eq!(pair.as_rule(), Rule::opening_hours);
    let mut pairs = pair.into_inner();
    let mut rules = Vec::new();

    while let Some(pair) = pairs.next() {
        rules.push(match pair.as_rule() {
            Rule::rule_sequence => build_rule_sequence(pair, rl::RuleOperator::Normal),
            Rule::any_rule_separator => build_rule_sequence(
                pairs.next().ok_or(Error::GrammarLogic {
                    rule: Rule::opening_hours,
                    invariant: "a separator is always followed by a rule",
                })?,
                build_any_rule_separator(pair)?,
            ),
            unexpected => {
                return Err(Error::GrammarUnexpectedToken { rule: Rule::opening_hours, unexpected })
            }
        }?)
    }

    Ok(rules)
}

fn build_rule_sequence(pair: Pair<Rule>, operator: rl::RuleOperator) -> Result<rl::RuleSequence> {
    debug_assert_eq!(pair.as_rule(), Rule::rule_sequence);
    let mut pairs = pair.into_inner();
    let root_pair = pairs.next().ok_or(err_empty(Rule::rule_sequence))?;
    let (day_selector, time_selector, extra_comment) = build_selector_sequence(root_pair)?;

    let (kind, comment) = pairs
        .next()
        .map(build_rules_modifier)
        .transpose()?
        .unwrap_or((rl::RuleKind::Open, None));

    let comment = comment
        .into_iter()
        .chain(extra_comment)
        .next()
        .unwrap_or_default()
        .into();

    Ok(rl::RuleSequence {
        day_selector,
        time_selector,
        kind,
        operator,
        comment,
    })
}

fn build_any_rule_separator(pair: Pair<Rule>) -> Result<rl::RuleOperator> {
    debug_assert_eq!(pair.as_rule(), Rule::any_rule_separator);

    let root_pair = pair
        .into_inner()
        .next()
        .ok_or(err_empty(Rule::any_rule_separator))?;

    match root_pair.as_rule() {
        Rule::normal_rule_separator => Ok(rl::RuleOperator::Normal),
        Rule::additional_rule_separator => Ok(rl::RuleOperator::Additional),
        Rule::fallback_rule_separator => Ok(rl::RuleOperator::Fallback),
        unexpected => {
            Err(Error::GrammarUnexpectedToken { rule: Rule::any_rule_separator, unexpected })
        }
    }
}

// --
// -- Rule modifier
// --

fn build_rules_modifier(pair: Pair<Rule>) -> Result<(rl::RuleKind, Option<String>)> {
    debug_assert_eq!(pair.as_rule(), Rule::rules_modifier);
    let mut pairs = pair.into_inner();

    let kind = pairs
        .next_if_rule(Rule::rules_modifier_enum)
        .map(build_rules_modifier_enum)
        .transpose()?
        .unwrap_or(rl::RuleKind::Open);

    let comment = pairs.next().map(build_comment).transpose()?;
    Ok((kind, comment))
}

fn build_rules_modifier_enum(pair: Pair<Rule>) -> Result<rl::RuleKind> {
    debug_assert_eq!(pair.as_rule(), Rule::rules_modifier_enum);

    let root_pair = pair
        .into_inner()
        .next()
        .ok_or(err_empty(Rule::rules_modifier_enum))?;

    match root_pair.as_rule() {
        Rule::rules_modifier_enum_closed => Ok(rl::RuleKind::Closed),
        Rule::rules_modifier_enum_open => Ok(rl::RuleKind::Open),
        Rule::rules_modifier_enum_unknown => Ok(rl::RuleKind::Unknown),
        unexpected => {
            Err(Error::GrammarUnexpectedToken { rule: Rule::rules_modifier_enum, unexpected })
        }
    }
}

// --
// -- Selectors
// --

fn build_selector_sequence(
    pair: Pair<Rule>,
) -> Result<(ds::DaySelector, ts::TimeSelector, Option<String>)> {
    debug_assert_eq!(pair.as_rule(), Rule::selector_sequence);
    let mut pairs = pair.into_inner();

    if pairs.next_if_rule(Rule::always_open).is_some() {
        return Ok(Default::default());
    }

    let (year, monthday, week, comment) = pairs
        .next_if_rule(Rule::wide_range_selectors)
        .map(build_wide_range_selectors)
        .transpose()?
        .unwrap_or_default();

    let (weekday, time) = pairs
        .next()
        .map(build_small_range_selectors)
        .transpose()?
        .unwrap_or_default();

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
    debug_assert_eq!(pair.as_rule(), Rule::wide_range_selectors);

    let mut year_selector = Vec::new();
    let mut monthday_selector = Vec::new();
    let mut week_selector = Vec::new();
    let mut comment = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::year_selector => year_selector = build_year_selector(pair)?,
            Rule::monthday_selector => monthday_selector = build_monthday_selector(pair)?,
            Rule::week_selector => week_selector = build_week_selector(pair)?,
            Rule::comment => comment = Some(build_comment(pair)?),
            unexpected => {
                return Err(Error::GrammarUnexpectedToken {
                    rule: Rule::wide_range_selectors,
                    unexpected,
                })
            }
        }
    }

    Ok((year_selector, monthday_selector, week_selector, comment))
}

fn build_small_range_selectors(
    pair: Pair<Rule>,
) -> Result<(Vec<ds::WeekDayRange>, Vec<ts::TimeSpan>)> {
    debug_assert_eq!(pair.as_rule(), Rule::small_range_selectors);

    let mut weekday_selector = Vec::new();
    let mut time_selector = Vec::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::weekday_selector => weekday_selector = build_weekday_selector(pair)?,
            Rule::time_selector => time_selector = build_time_selector(pair)?,
            unexpected => {
                return Err(Error::GrammarUnexpectedToken {
                    rule: Rule::wide_range_selectors,
                    unexpected,
                })
            }
        }
    }

    Ok((weekday_selector, time_selector))
}

// --
// -- Time selector
// --

fn build_time_selector(pair: Pair<Rule>) -> Result<Vec<ts::TimeSpan>> {
    debug_assert_eq!(pair.as_rule(), Rule::time_selector);
    pair.into_inner().map(build_timespan).collect()
}

fn build_timespan(pair: Pair<Rule>) -> Result<ts::TimeSpan> {
    debug_assert_eq!(pair.as_rule(), Rule::timespan);
    let mut pairs = pair.into_inner();
    let mut repeats = None;
    let start = build_time(pairs.next().ok_or(err_empty(Rule::timespan))?)?;

    let (mut open_end, end) = match pairs.next() {
        None => {
            return Err(Error::Unsupported("point in time"));
        }
        Some(pair) if pair.as_rule() == Rule::timespan_plus => {
            // TODO: opening_hours.js handles this better: it will set the
            //       state to unknown and add a warning comment.
            (true, ts::Time::Fixed(ExtendedTime::MIDNIGHT_24))
        }
        Some(pair) => (false, build_extended_time(pair)?),
    };

    if let Some(pair_repetition) = pairs.next() {
        match pair_repetition.as_rule() {
            Rule::timespan_plus => open_end = true,
            Rule::minute => repeats = Some(build_minute(pair_repetition)?),
            Rule::hour_minutes => repeats = Some(build_hour_minutes_as_duration(pair_repetition)?),
            unexpected => {
                return Err(Error::GrammarUnexpectedToken { rule: Rule::timespan, unexpected })
            }
        }
    }

    debug_assert!(pairs.next().is_none());
    Ok(ts::TimeSpan { range: start..end, repeats, open_end })
}

fn build_time(pair: Pair<Rule>) -> Result<ts::Time> {
    debug_assert_eq!(pair.as_rule(), Rule::time);
    let root_pair = pair.into_inner().next().ok_or(err_empty(Rule::time))?;

    Ok(match root_pair.as_rule() {
        Rule::hour_minutes => ts::Time::Fixed(build_hour_minutes(root_pair)?),
        Rule::variable_time => ts::Time::Variable(build_variable_time(root_pair)?),
        unexpected => return Err(Error::GrammarUnexpectedToken { rule: Rule::time, unexpected }),
    })
}

fn build_extended_time(pair: Pair<Rule>) -> Result<ts::Time> {
    debug_assert_eq!(pair.as_rule(), Rule::extended_time);

    let root_pair = pair
        .into_inner()
        .next()
        .ok_or(err_empty(Rule::extended_time))?;

    match root_pair.as_rule() {
        Rule::extended_hour_minutes => build_extended_hour_minutes(root_pair).map(ts::Time::Fixed),
        Rule::variable_time => build_variable_time(root_pair).map(ts::Time::Variable),
        unexpected => Err(Error::GrammarUnexpectedToken { rule: Rule::extended_time, unexpected }),
    }
}

fn build_variable_time(pair: Pair<Rule>) -> Result<ts::VariableTime> {
    debug_assert_eq!(pair.as_rule(), Rule::variable_time);
    let mut pairs = pair.into_inner();
    let event = build_event(pairs.next().ok_or(err_empty(Rule::variable_time))?)?;

    let offset = {
        if let Some(sign_pair) = pairs.next() {
            let sign = build_plus_or_minus(sign_pair)?;

            let hour_minutes_pair = pairs.next().ok_or(Error::GrammarLogic {
                rule: Rule::variable_time,
                invariant: "a sign is always followed by hours and minutes",
            })?;

            let mins: i16 = build_hour_minutes(hour_minutes_pair)?
                .mins_from_midnight()
                .try_into()
                .map_err(|_| Error::GrammarLogic {
                    rule: Rule::variable_time,
                    invariant: "daily number of minutes fits in an i16",
                })?;

            match sign {
                Sign::Pos => mins,
                Sign::Neg => -mins,
            }
        } else {
            0
        }
    };

    Ok(ts::VariableTime { event, offset })
}

fn build_event(pair: Pair<Rule>) -> Result<ts::TimeEvent> {
    debug_assert_eq!(pair.as_rule(), Rule::event);
    let root_pair = pair.into_inner().next().ok_or(err_empty(Rule::event))?;

    match root_pair.as_rule() {
        Rule::dawn => Ok(ts::TimeEvent::Dawn),
        Rule::sunrise => Ok(ts::TimeEvent::Sunrise),
        Rule::sunset => Ok(ts::TimeEvent::Sunset),
        Rule::dusk => Ok(ts::TimeEvent::Dusk),
        unexpected => Err(Error::GrammarUnexpectedToken { rule: Rule::event, unexpected }),
    }
}

// --
// -- WeekDay selector
// --

fn build_weekday_selector(pair: Pair<Rule>) -> Result<Vec<ds::WeekDayRange>> {
    debug_assert_eq!(pair.as_rule(), Rule::weekday_selector);
    let mut ranges = Vec::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::weekday_sequence => {
                for pair in pair.into_inner() {
                    ranges.push(build_weekday_range(pair)?)
                }
            }
            Rule::holiday_sequence => {
                for pair in pair.into_inner() {
                    ranges.push(build_holiday(pair)?)
                }
            }
            unexpected => {
                return Err(Error::GrammarUnexpectedToken {
                    rule: Rule::weekday_selector,
                    unexpected,
                })
            }
        }
    }

    Ok(ranges)
}

fn build_weekday_range(pair: Pair<Rule>) -> Result<ds::WeekDayRange> {
    debug_assert_eq!(pair.as_rule(), Rule::weekday_range);
    let mut pairs = pair.into_inner();
    let start = build_wday(pairs.next().ok_or(err_empty(Rule::weekday_range))?)?;

    let end = pairs
        .next_if_rule(Rule::wday)
        .map(build_wday)
        .transpose()?
        .unwrap_or(start);

    let mut nth_from_start = [false; 5];
    let mut nth_from_end = [false; 5];

    while let Some(pair_nth_entry) = pairs.next_if_rule(Rule::nth_entry) {
        let (sign, indices) = build_nth_entry(pair_nth_entry)?;

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
    debug_assert_eq!(pair.as_rule(), Rule::holiday);
    let mut pairs = pair.into_inner();

    let kind = match pairs.next().ok_or(err_empty(Rule::holiday))?.as_rule() {
        Rule::public_holiday => ds::HolidayKind::Public,
        Rule::school_holiday => ds::HolidayKind::School,
        unexpected => {
            return Err(Error::GrammarUnexpectedToken { rule: Rule::holiday, unexpected })
        }
    };

    let offset = pairs.next().map(build_day_offset).unwrap_or(Ok(0))?;
    Ok(ds::WeekDayRange::Holiday { kind, offset })
}

fn build_nth_entry(pair: Pair<Rule>) -> Result<(Sign, RangeInclusive<u8>)> {
    debug_assert_eq!(pair.as_rule(), Rule::nth_entry);
    let mut pairs = pair.into_inner();

    let sign = {
        if pairs.next_if_rule(Rule::nth_minus).is_some() {
            Sign::Neg
        } else {
            Sign::Pos
        }
    };

    let start = build_nth(pairs.next().ok_or(Error::GrammarLogic {
        rule: Rule::nth_entry,
        invariant: "a sign is always followed by a number",
    })?)?;

    let end = pairs.next().map(build_nth).transpose()?.unwrap_or(start);
    Ok((sign, start..=end))
}

fn build_nth(pair: Pair<Rule>) -> Result<u8> {
    debug_assert_eq!(pair.as_rule(), Rule::nth);

    pair.as_str().parse().map_err(|_| Error::GrammarLogic {
        rule: Rule::nth,
        invariant: "must be valid number for 1 to 5",
    })
}

fn build_day_offset(pair: Pair<Rule>) -> Result<i16> {
    debug_assert_eq!(pair.as_rule(), Rule::day_offset);
    let mut pairs = pair.into_inner();
    let sign = build_plus_or_minus(pairs.next().ok_or(err_empty(Rule::day_offset))?)?;

    let val_abs = build_positive_number(pairs.next().ok_or(Error::GrammarLogic {
        rule: Rule::day_offset,
        invariant: "a sign is always followed by a number",
    })?)?;

    Ok(match sign {
        Sign::Pos => val_abs,
        Sign::Neg => -val_abs,
    })
}

// --
// -- Week selector
// --

fn build_week_selector(pair: Pair<Rule>) -> Result<Vec<ds::WeekRange>> {
    debug_assert_eq!(pair.as_rule(), Rule::week_selector);
    pair.into_inner().map(build_week).collect()
}

fn build_week(pair: Pair<Rule>) -> Result<ds::WeekRange> {
    debug_assert_eq!(pair.as_rule(), Rule::week);
    let mut rules = pair.into_inner();
    let start = build_weeknum(rules.next().ok_or(err_empty(Rule::week))?)?;

    let end = rules
        .next()
        .map(build_weeknum)
        .transpose()?
        .unwrap_or(start);

    let step = rules
        .next()
        .map(build_positive_number)
        .transpose()?
        .unwrap_or(1);

    let step = step
        .try_into()
        .map_err(|_| Error::Overflow { value: step, expected_bounds: 0i16..=255i16 })?;

    ds::WeekRange::new(start..=end, step).ok_or(Error::InvertedWeekRange { start, end, step })
}

// --
// -- Month selector
// --

fn build_monthday_selector(pair: Pair<Rule>) -> Result<Vec<ds::MonthdayRange>> {
    debug_assert_eq!(pair.as_rule(), Rule::monthday_selector);
    pair.into_inner().map(build_monthday_range).collect()
}

fn build_monthday_range(pair: Pair<Rule>) -> Result<ds::MonthdayRange> {
    debug_assert_eq!(pair.as_rule(), Rule::monthday_range);
    let mut pairs = pair.into_inner();
    let mut first_pair = pairs.next().ok_or(err_empty(Rule::monthday_range))?;

    let year = {
        if first_pair.as_rule() == Rule::year {
            let year = build_year(first_pair)?;

            first_pair = pairs.next().ok_or(Error::GrammarLogic {
                rule: Rule::monthday_range,
                invariant: "cannot contain just a year",
            })?;

            Some(year)
        } else {
            None
        }
    };

    match first_pair.as_rule() {
        Rule::month => {
            let start = build_month(first_pair)?;
            let end = pairs.next().map(build_month).transpose()?.unwrap_or(start);
            Ok(ds::MonthdayRange::Month { year, range: start..=end })
        }
        Rule::date_from => {
            let start = build_date_from(first_pair)?;

            let start_offset = pairs
                .next_if_rule(Rule::date_offset)
                .map(build_date_offset)
                .transpose()?
                .unwrap_or_default();

            let Some(pair_end) = pairs.next() else {
                return Ok(ds::MonthdayRange::Date {
                    start: (start, start_offset),
                    end: (start, start_offset),
                });
            };

            let end = match pair_end.as_rule() {
                Rule::date_to => build_date_to(pair_end, start)?,
                Rule::monthday_range_plus => {
                    if start.year().is_some() {
                        ds::Date::ymd(31, ds::Month::December, Year(9999))
                    } else {
                        ds::Date::md(31, ds::Month::December)
                    }
                }
                unexpected => {
                    return Err(Error::GrammarUnexpectedToken {
                        rule: Rule::monthday_range,
                        unexpected,
                    })
                }
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
        unexpected => Err(Error::GrammarUnexpectedToken { rule: Rule::monthday_range, unexpected }),
    }
}

fn build_date_offset(pair: Pair<Rule>) -> Result<ds::DateOffset> {
    debug_assert_eq!(pair.as_rule(), Rule::date_offset);
    let mut pairs = pair.into_inner();

    let wday_offset = {
        if let Some(pair_sign) = pairs.next_if_rule(Rule::plus_or_minus) {
            let sign = build_plus_or_minus(pair_sign)?;

            let wday = build_wday(pairs.next().ok_or(Error::GrammarLogic {
                rule: Rule::date_offset,
                invariant: "a sign is always followed by a wday",
            })?)?;

            match sign {
                Sign::Pos => ds::WeekDayOffset::Next(wday),
                Sign::Neg => ds::WeekDayOffset::Prev(wday),
            }
        } else {
            ds::WeekDayOffset::None
        }
    };

    let day_offset = pairs.next().map(build_day_offset).unwrap_or(Ok(0))?;
    Ok(ds::DateOffset { wday_offset, day_offset })
}

fn build_date_from(pair: Pair<Rule>) -> Result<ds::Date> {
    debug_assert_eq!(pair.as_rule(), Rule::date_from);
    let mut pairs = pair.into_inner();
    let year = pairs.next_if_rule(Rule::year).map(build_year).transpose()?;

    let pair_month_or_variable = pairs.next().ok_or(Error::GrammarLogic {
        rule: Rule::date_from,
        invariant: "must have a month component",
    })?;

    if pair_month_or_variable.as_rule() == Rule::variable_date {
        return Ok(ds::Date::Easter { year });
    }

    let month = build_month(pair_month_or_variable)?;

    let pair_day = pairs.next().ok_or(Error::GrammarLogic {
        rule: Rule::date_from,
        invariant: "must have a daynum or wday component",
    })?;

    match pair_day.as_rule() {
        Rule::daynum => Ok(ds::Date::Fixed { year, month, day: build_daynum(pair_day)? }),
        Rule::wday => {
            let weekday = build_wday(pair_day)?;

            let nth_sign = {
                if pairs.next_if_rule(Rule::nth_minus).is_some() {
                    -1
                } else {
                    1
                }
            };

            let nth: i8 = (pairs.next())
                .map(build_nth)
                .transpose()?
                .ok_or(Error::GrammarLogic {
                    rule: Rule::date_from,
                    invariant: "a sign is always followed by a number",
                })?
                .try_into()
                .map_err(|_| Error::GrammarLogic {
                    rule: Rule::date_from,
                    invariant: "must be a valid number between 1 and 5",
                })?;

            Ok(ds::Date::Weekday { year, month, wday: weekday, nth: nth_sign * nth })
        }
        unexpected => Err(Error::GrammarUnexpectedToken { rule: Rule::date_from, unexpected }),
    }
}

fn build_date_to(pair: Pair<Rule>, from: ds::Date) -> Result<ds::Date> {
    debug_assert_eq!(pair.as_rule(), Rule::date_to);
    let pair = pair.into_inner().next().ok_or(err_empty(Rule::date_to))?;

    match pair.as_rule() {
        Rule::date_from => build_date_from(pair),
        Rule::daynum => {
            let daynum = build_daynum(pair)?;

            match from {
                ds::Date::Easter { .. } => {
                    // NOTE: this is actually not a specified constraint, but allowing this could
                    // be super ambiguous anyway as the resulting end month could vary depending on
                    // current year's easter date.
                    Err(Error::Unsupported("Easter followed by a day number"))
                }
                ds::Date::Weekday { year, month, .. } => {
                    Ok(ds::Date::Fixed { year, month, day: daynum })
                }
                ds::Date::Fixed { mut year, mut month, day } => {
                    if day > daynum {
                        month = month.next();

                        if month == ds::Month::January {
                            if let Some(x) = year.as_mut() {
                                **x += 1
                            }
                        }
                    }

                    Ok(ds::Date::Fixed { year, month, day: daynum })
                }
            }
        }
        unexpected => Err(Error::GrammarUnexpectedToken { rule: Rule::date_to, unexpected }),
    }
}

// --
// -- Year selector
// --

fn build_year_selector(pair: Pair<Rule>) -> Result<Vec<ds::YearRange>> {
    debug_assert_eq!(pair.as_rule(), Rule::year_selector);
    pair.into_inner().map(build_year_range).collect()
}

fn build_year_range(pair: Pair<Rule>) -> Result<ds::YearRange> {
    debug_assert_eq!(pair.as_rule(), Rule::year_range);
    let mut rules = pair.into_inner();
    let start = build_year(rules.next().ok_or(err_empty(Rule::year_range))?)?;

    let end = rules
        .next()
        .map(|pair| match pair.as_rule() {
            Rule::year => build_year(pair),
            Rule::year_range_plus => Ok(Year(9999)),
            unexpected => Err(Error::GrammarUnexpectedToken { rule: Rule::year_range, unexpected }),
        })
        .transpose()?
        .unwrap_or(start);

    let step = rules
        .next()
        .map(build_positive_number)
        .transpose()?
        .unwrap_or(1)
        .unsigned_abs();

    ds::YearRange::new(start..=end, step).ok_or(Error::InvertedYearRange { start, end, step })
}

// --
// -- Basic elements
// --

fn build_plus_or_minus(pair: Pair<Rule>) -> Result<Sign> {
    debug_assert_eq!(pair.as_rule(), Rule::plus_or_minus);

    let pair = pair
        .into_inner()
        .next()
        .ok_or(err_empty(Rule::plus_or_minus))?;

    match pair.as_rule() {
        Rule::plus => Ok(Sign::Pos),
        Rule::minus => Ok(Sign::Neg),
        unexpected => Err(Error::GrammarUnexpectedToken { rule: Rule::plus_or_minus, unexpected }),
    }
}

fn build_minute(pair: Pair<Rule>) -> Result<Duration> {
    debug_assert_eq!(pair.as_rule(), Rule::minute);

    pair.as_str()
        .parse()
        .map_err(|_| Error::GrammarLogic {
            rule: Rule::minute,
            invariant: "must be a valid number",
        })
        .map(Duration::minutes)
}

fn build_hour_minutes(pair: Pair<Rule>) -> Result<ExtendedTime> {
    debug_assert_eq!(pair.as_rule(), Rule::hour_minutes);
    let mut pairs = pair.into_inner();

    let Some(hour_rule) = pairs.next() else {
        return Ok(ExtendedTime::MIDNIGHT_24);
    };

    let hour = hour_rule
        .as_str()
        .parse()
        .map_err(|_| Error::GrammarLogic {
            rule: Rule::hour,
            invariant: "must be a valid number",
        })?;

    let minutes = pairs
        .next()
        .ok_or(Error::GrammarLogic {
            rule: Rule::hour_minutes,
            invariant: "hour must be followed by minutes",
        })?
        .as_str()
        .parse()
        .map_err(|_| Error::GrammarLogic {
            rule: Rule::minute,
            invariant: "must be a valid number",
        })?;

    ExtendedTime::new(hour, minutes).ok_or(Error::InvalidExtendedTime { hour, minutes })
}

fn build_extended_hour_minutes(pair: Pair<Rule>) -> Result<ExtendedTime> {
    debug_assert_eq!(pair.as_rule(), Rule::extended_hour_minutes);
    let mut pairs = pair.into_inner();

    let hour = pairs
        .next()
        .ok_or(err_empty(Rule::extended_hour_minutes))?
        .as_str()
        .parse()
        .map_err(|_| Error::GrammarLogic {
            rule: Rule::extended_hour,
            invariant: "must be a valid number",
        })?;

    let minutes = pairs
        .next()
        .ok_or(Error::GrammarLogic {
            rule: Rule::extended_hour_minutes,
            invariant: "hour must be followed by minutes",
        })?
        .as_str()
        .parse()
        .map_err(|_| Error::GrammarLogic {
            rule: Rule::minute,
            invariant: "must be a valid number",
        })?;

    ExtendedTime::new(hour, minutes).ok_or(Error::InvalidExtendedTime { hour, minutes })
}

fn build_hour_minutes_as_duration(pair: Pair<Rule>) -> Result<Duration> {
    debug_assert_eq!(pair.as_rule(), Rule::hour_minutes);
    let mut pairs = pair.into_inner();

    let hour = pairs
        .next()
        .ok_or(err_empty(Rule::hour_minutes))?
        .as_str()
        .parse()
        .map_err(|_| Error::GrammarLogic {
            rule: Rule::hour,
            invariant: "must be a valid number",
        })?;

    let minutes = pairs
        .next()
        .ok_or(Error::GrammarLogic {
            rule: Rule::hour_minutes,
            invariant: "hour must be followed by minutes",
        })?
        .as_str()
        .parse()
        .map_err(|_| Error::GrammarLogic {
            rule: Rule::minute,
            invariant: "must be a valid number",
        })?;

    Ok(Duration::hours(hour) + Duration::minutes(minutes))
}

fn build_wday(pair: Pair<Rule>) -> Result<ds::Weekday> {
    debug_assert_eq!(pair.as_rule(), Rule::wday);
    let pair = pair.into_inner().next().ok_or(err_empty(Rule::wday))?;

    match pair.as_rule() {
        Rule::sunday => Ok(ds::Weekday::Sun),
        Rule::monday => Ok(ds::Weekday::Mon),
        Rule::tuesday => Ok(ds::Weekday::Tue),
        Rule::wednesday => Ok(ds::Weekday::Wed),
        Rule::thursday => Ok(ds::Weekday::Thu),
        Rule::friday => Ok(ds::Weekday::Fri),
        Rule::saturday => Ok(ds::Weekday::Sat),
        unexpected => Err(Error::GrammarUnexpectedToken { rule: Rule::wday, unexpected }),
    }
}

fn build_daynum(pair: Pair<Rule>) -> Result<u8> {
    debug_assert_eq!(pair.as_rule(), Rule::daynum);

    let daynum = pair.as_str().parse().map_err(|_| Error::GrammarLogic {
        rule: Rule::daynum,
        invariant: "must be a valid number",
    })?;

    if daynum == 0 {
        #[cfg(feature = "log")]
        log::warn!("Found day number 0 in opening hours: specify the 1st or 31st instead.");
        return Ok(1);
    }

    if daynum > 31 {
        #[cfg(feature = "log")]
        log::warn!("Found day number {daynum} in opening hours");
        return Ok(31);
    }

    Ok(daynum)
}

fn build_weeknum(pair: Pair<Rule>) -> Result<WeekNum> {
    debug_assert_eq!(pair.as_rule(), Rule::weeknum);

    pair.as_str()
        .parse()
        .map_err(|_| Error::GrammarLogic {
            rule: Rule::weeknum,
            invariant: "must be a valid number",
        })
        .map(WeekNum)
}

fn build_month(pair: Pair<Rule>) -> Result<ds::Month> {
    debug_assert_eq!(pair.as_rule(), Rule::month);
    let pair = pair.into_inner().next().ok_or(err_empty(Rule::month))?;

    match pair.as_rule() {
        Rule::january => Ok(ds::Month::January),
        Rule::february => Ok(ds::Month::February),
        Rule::march => Ok(ds::Month::March),
        Rule::april => Ok(ds::Month::April),
        Rule::may => Ok(ds::Month::May),
        Rule::june => Ok(ds::Month::June),
        Rule::july => Ok(ds::Month::July),
        Rule::august => Ok(ds::Month::August),
        Rule::september => Ok(ds::Month::September),
        Rule::october => Ok(ds::Month::October),
        Rule::november => Ok(ds::Month::November),
        Rule::december => Ok(ds::Month::December),
        unexpected => Err(Error::GrammarUnexpectedToken { rule: Rule::month, unexpected }),
    }
}

fn build_year(pair: Pair<Rule>) -> Result<Year> {
    debug_assert_eq!(pair.as_rule(), Rule::year);

    pair.as_str()
        .parse()
        .map_err(|_| Error::GrammarLogic {
            rule: Rule::year,
            invariant: "must be a valid number",
        })
        .map(Year)
}

fn build_positive_number(pair: Pair<Rule>) -> Result<i16> {
    debug_assert_eq!(pair.as_rule(), Rule::positive_number);

    let val = pair.as_str().parse().map_err(|_| Error::GrammarLogic {
        rule: Rule::positive_number,
        invariant: "must be a valid 16 bits number",
    })?;

    debug_assert!(val >= 0);
    Ok(val)
}

fn build_comment(pair: Pair<Rule>) -> Result<String> {
    debug_assert_eq!(pair.as_rule(), Rule::comment);

    pair.into_inner()
        .next()
        .ok_or(err_empty(Rule::comment))
        .map(build_comment_inner)
}

fn build_comment_inner(pair: Pair<Rule>) -> String {
    debug_assert_eq!(pair.as_rule(), Rule::comment_inner);
    pair.as_str().to_string()
}

// --
// -- Mics
// --

enum Sign {
    Neg,
    Pos,
}

/// Commonly built errors
fn err_empty(rule: Rule) -> Error {
    Error::GrammarLogic { rule, invariant: "cannot be empty" }
}

/// Extra helpers for pairs iterator.
trait PairsIterExtension {
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
