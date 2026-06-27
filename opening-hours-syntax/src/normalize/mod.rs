pub(crate) mod bounded;
pub(crate) mod canonical_date;
pub(crate) mod canonical_time;
pub(crate) mod paving;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use crate::RuleKind;
use crate::normalize::canonical_date::{CanonicalDate, CanonicalDateSelector};
use crate::normalize::canonical_time::{TimeRules, no_overlap_with_next_day, normalize_time_rules};
use crate::normalize::paving::Paving;
use crate::rules::day::DaySelector;
use crate::rules::{RuleOperator, RuleSequence};

/// Consume as much rules as possible to be converted into a CanonicalDate.
pub(crate) fn drain_ruleseq_into_canonical(rules: &mut VecDeque<RuleSequence>) -> CanonicalDate {
    let mut canonical = CanonicalDate::default();

    #[allow(clippy::result_large_err)]
    while let Some(rule) = rules.pop_front() {
        if rule.operator == RuleOperator::Fallback || !no_overlap_with_next_day(&rule.time_selector)
        {
            rules.push_front(rule);
            return canonical;
        }

        let Ok(selector) = CanonicalDateSelector::try_from(&rule.day_selector) else {
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

/// Transform a CanonicalDate into a RuleSequence.
pub(crate) fn canonical_to_ruleseq(canonical: CanonicalDate) -> Vec<RuleSequence> {
    let mut result = Vec::new();
    let mut canonical_remaining = canonical.map(normalize_time_rules);

    // Insert rules in the following orders:
    #[allow(clippy::type_complexity)]
    let pop_priority_order: [Box<dyn Fn(&TimeRules) -> bool>; _] = [
        // 1. only has "open" time ranges
        Box::new(|s| !s.is_empty() && s.iter().all(|s| s.0.0 == RuleKind::Open)),
        // 2. starts with an "open" time range
        Box::new(|s| s.first().map(|s| s.0.0 == RuleKind::Open).unwrap_or(false)),
        // 3. only has "unknown" time ranges
        Box::new(|s| !s.is_empty() && s.iter().all(|s| s.0.0 == RuleKind::Unknown)),
        // 4. starts with an unknown time range
        Box::new(|s| {
            s.first()
                .map(|s| s.0.0 == RuleKind::Unknown)
                .unwrap_or(false)
        }),
        // 5. starts with a close time range
        Box::new(|s| {
            !s.is_empty()
                && s.iter()
                    .any(|s| s.0.0 != RuleKind::Unknown || !s.0.1.is_empty())
        }),
    ];

    for pop_priority in pop_priority_order {
        while let Some((slots, selector)) = canonical_remaining.pop_filter(&pop_priority) {
            let day_selector: DaySelector = selector.into();
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
