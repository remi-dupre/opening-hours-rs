pub(crate) mod canonical;
pub(crate) mod canonical_time;
pub(crate) mod frame;
pub(crate) mod paving;

use std::collections::VecDeque;

use crate::normalize::canonical::{CanonicalDaySelector, OrderedWeekday};
use crate::normalize::canonical_time::{normalize_time_rules, TimeRules};
use crate::normalize::frame::Frame;
use crate::normalize::paving::{EmptyPavingSelector, Paving, Paving4D, UnpackFromBack};
use crate::rules::day::{DaySelector, Month, WeekNum, Year};
use crate::rules::time::TimeSelector;
use crate::rules::{RuleOperator, RuleSequence};
use crate::RuleKind;

use self::canonical::{Canonical, CanonicalSelector, MakeCanonical};
use self::paving::SelectorCompression;

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

/// Convert a rule sequence to a n-dim selector.
pub(crate) fn ruleseq_to_selector(rs: &RuleSequence) -> Option<CanonicalSelector> {
    let ds = &rs.day_selector;

    let selector = EmptyPavingSelector
        .dim_front(MakeCanonical::try_from_iterator(&rs.time_selector.spans)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.year)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.monthday)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.week)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.weekday)?);

    Some(selector)
}

pub(crate) type Canonical2 =
    Paving4D<Frame<OrderedWeekday>, Frame<WeekNum>, Frame<Month>, Frame<Year>, TimeRules>;

pub(crate) fn partialytocanonical2(rules: &mut VecDeque<RuleSequence>) -> Canonical2 {
    let mut canonical = Canonical2::default();

    #[allow(clippy::result_large_err)]
    while let Some(rule) = rules.pop_front() {
        if rule.operator == RuleOperator::Fallback {
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
        Box::new(|s| !s.is_empty() && s.iter().all(|s| s.0 .0 == RuleKind::Closed)),
        Box::new(|s| !s.is_empty()),
    ];

    for pop_priority in pop_priority_order {
        while let Some((slots, mut selector)) = canonical_remaining.pop_filter(&pop_priority) {
            selector.fill_holes(|candidate| {
                // If the date domain is covered by any rule that is not closed, this means that a
                // created range would be overriden anyway.
                let will_be_overriden = canonical_remaining.check_predicate(candidate, |s| {
                    s.iter().any(|((kind, _), _)| *kind != RuleKind::Closed)
                });

                // It is also okay to override a range that contains the exact time values.
                println!("{candidate:#?}");
                let overrides_same_value = canonical_before.is_val(candidate, &slots);
                will_be_overriden || overrides_same_value
            });

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

/// Convert a canonical paving back into a rules sequence.
pub(crate) fn canonical_to_seq(canonical: Canonical) -> impl Iterator<Item = RuleSequence> {
    // Keep track of the days that have already been outputed. This allows to
    // use an additional rule only when necessary.
    let mut days_covered = Paving4D::default();

    // Parts of this paving will be removed until it is empty
    let mut canonical_remaining = canonical.clone();

    core::iter::from_fn(move || {
        // Extract open periods first, then unknowns
        let ((kind, comment), mut selector) = [RuleKind::Open, RuleKind::Unknown, RuleKind::Closed]
            .into_iter()
            .find_map(|target_kind| {
                canonical_remaining.pop_filter(|(kind, comment)| {
                    *kind == target_kind && (target_kind != RuleKind::Closed || !comment.is_empty())
                })
            })?;

        // Merge consecutive intervals as much as possible if the hole between
        // two consecutive intervals was covered with the same value during a
        // previous extraction.
        selector.fill_holes({
            let canonical = &canonical;
            let val = (kind, comment.clone());
            move |candidate| canonical.is_val(candidate, &val)
        });

        let (day_selector, rgs_time) = selector.into_unpack_back();

        // If the current sequence doesn't cover any day with any time range
        // already defined, we can use a normal rule operator which is more
        // common. Otherwise, fallback to an additional rule operator, which
        // has a more predictable semantic.
        let operator = {
            let no_day_overlap = days_covered.is_val(&day_selector, &false);

            if no_day_overlap {
                RuleOperator::Normal
            } else {
                RuleOperator::Additional
            }
        };

        // Mark the days as (partialy) covered
        days_covered.set(&day_selector, &true);

        // Extract remaining dimensions
        let (rgs_weekday, day_selector) = day_selector.into_unpack_front();
        let (rgs_week, day_selector) = day_selector.into_unpack_front();
        let (rgs_monthday, day_selector) = day_selector.into_unpack_front();
        let (rgs_year, EmptyPavingSelector) = day_selector.into_unpack_front();

        let day_selector = DaySelector {
            year: MakeCanonical::into_selector(rgs_year, true),
            monthday: MakeCanonical::into_selector(rgs_monthday, true),
            week: MakeCanonical::into_selector(rgs_week, true),
            weekday: MakeCanonical::into_selector(rgs_weekday, true),
        };

        let time_selector = TimeSelector {
            spans: MakeCanonical::into_selector(rgs_time, false),
        };

        Some(RuleSequence {
            day_selector,
            time_selector,
            kind,
            operator,
            comment,
        })
    })
}
