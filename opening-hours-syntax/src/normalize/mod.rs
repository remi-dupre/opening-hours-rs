pub(crate) mod bounded;
pub(crate) mod canonical_date;
pub(crate) mod canonical_time;
pub(crate) mod paving;

use std::collections::VecDeque;

use crate::normalize::bounded::Frame;
use crate::normalize::canonical_date::{CanonicalDaySelector, MakeCanonical};
use crate::normalize::canonical_time::{
    can_overlap_with_next_day, normalize_time_rules, TimeRules,
};
use crate::normalize::paving::{EmptyPavingSelector, Paving, Paving4D};
use crate::rules::day::{DaySelector, Month, WeekNum, Year};
use crate::rules::{RuleOperator, RuleSequence};
use crate::util::weekday::OrderedWeekday;
use crate::RuleKind;

// --
// -- Normalization Logic
// --

/// Convert day fields of a rule sequence to a n-dim selector.
pub(crate) fn ruleseq_to_day_selector(rs: &RuleSequence) -> Option<CanonicalDaySelector> {
    let ds = &rs.day_selector;

    let selector = EmptyPavingSelector
        .dim_front(MakeCanonical::try_from_iterator(&ds.year)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.monthday)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.week)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.weekday)?);

    Some(selector)
}

pub(crate) type Canonical2 =
    Paving4D<Frame<OrderedWeekday>, WeekNum, Frame<Month>, Year, TimeRules>;

pub(crate) fn partialytocanonical2(rules: &mut VecDeque<RuleSequence>) -> Canonical2 {
    let mut canonical = Canonical2::default();

    #[allow(clippy::result_large_err)]
    while let Some(rule) = rules.pop_front() {
        if rule.operator == RuleOperator::Fallback || can_overlap_with_next_day(&rule.time_selector)
        {
            rules.push_front(rule);
            return canonical;
        }

        let Some(selector) = ruleseq_to_day_selector(&rule) else {
            rules.push_front(rule);
            return canonical;
        };

        let new_val = ((rule.kind, rule.comment), rule.time_selector);

        if rule.operator == RuleOperator::Normal && rule.kind != RuleKind::Closed {
            canonical.set(&selector, &vec![new_val]);
        } else {
            canonical.update(&selector, |content| content.push(new_val.clone()))
        }
    }

    canonical
}

pub(crate) fn canonical_to_seq2(canonical: Canonical2) -> Vec<RuleSequence> {
    let canonical_before = canonical.map(normalize_time_rules);
    let mut result = Vec::new();
    let mut canonical_remaining = canonical_before.clone();

    // Insert rules in the following orders:
    #[allow(clippy::type_complexity)]
    let pop_priority_order: [Box<dyn Fn(&TimeRules) -> bool>; _] = [
        // 1. only has "open" time ranges
        Box::new(|s| !s.is_empty() && s.iter().all(|s| s.0 .0 == RuleKind::Open)),
        // 2. starts with an "open" time range
        Box::new(|s| s.first().map(|s| s.0 .0 == RuleKind::Open).unwrap_or(false)),
        // 3. only has "unknown" time ranges
        Box::new(|s| !s.is_empty() && s.iter().all(|s| s.0 .0 == RuleKind::Unknown)),
        // 4. starts with an unknown time range
        Box::new(|s| {
            s.first()
                .map(|s| s.0 .0 == RuleKind::Unknown)
                .unwrap_or(false)
        }),
        // 5. starts with a close time range
        Box::new(|s| {
            !s.is_empty()
                && s.iter()
                    .any(|s| s.0 .0 != RuleKind::Unknown || !s.0 .1.is_empty())
        }),
    ];

    for pop_priority in pop_priority_order {
        while let Some((slots, selector)) = canonical_remaining.pop_filter(&pop_priority) {
            // Unpack ranges
            let (rgs_weekday, selector) = selector.into_unpack_front();
            let (rgs_week, selector) = selector.into_unpack_front();
            let (rgs_monthday, selector) = selector.into_unpack_front();
            let (rgs_year, EmptyPavingSelector) = selector.into_unpack_front();

            let day_selector = DaySelector {
                year: MakeCanonical::into_selector(rgs_year, true),
                monthday: MakeCanonical::into_selector(rgs_monthday, true),
                week: MakeCanonical::into_selector(rgs_week, true),
                weekday: MakeCanonical::into_selector(rgs_weekday, true),
            };

            let mut first_time_component = true;

            for ((kind, comment), time_selector) in slots {
                let operator = {
                    if first_time_component {
                        RuleOperator::Normal
                    } else {
                        RuleOperator::Additional
                    }
                };

                result.push(RuleSequence {
                    day_selector: day_selector.clone(),
                    time_selector,
                    kind,
                    operator,
                    comment,
                });

                first_time_component = false;
            }
        }
    }

    result
}
