pub(crate) mod canonical;
pub(crate) mod frame;
pub(crate) mod paving;

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::normalize::canonical::OrderedWeekday;
use crate::normalize::frame::{Bounded, Frame};
use crate::normalize::paving::{
    EmptyPavingSelector, Paving, Paving3D, Paving4D, Paving5D, UnpackFromBack,
};
use crate::rules::day::{DaySelector, Month, WeekNum, Year};
use crate::rules::time::{TimeSelector, TimeSpan};
use crate::rules::{RuleOperator, RuleSequence};
use crate::RuleKind;

use self::canonical::{Canonical, CanonicalSelector, MakeCanonical};
use self::paving::SelectorCompression;

// --
// -- Normalization Logic
// --

/// Convert a rule sequence to a n-dim selector.
pub(crate) fn ruleseq_to_selector(rs: &RuleSequence) -> Option<CanonicalSelector> {
    let ds = &rs.day_selector;

    let selector = EmptyPavingSelector
        .dim_front(MakeCanonical::try_from_iterator(&rs.time_selector.time)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.year)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.monthday)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.week)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.weekday)?);

    Some(selector)
}

pub(crate) type CanonicalSlot =
    Paving4D<Frame<OrderedWeekday>, Frame<WeekNum>, Frame<Month>, Frame<Year>, Vec<TimeSpan>>;

type Canonical2 = HashMap<(RuleKind, Arc<str>), CanonicalSlot>;

pub(crate) fn partialytocanonical2(
    rules: &mut VecDeque<RuleSequence>,
) -> HashMap<(RuleKind, Arc<str>), CanonicalSlot> {
    let mut canonical: HashMap<(RuleKind, Arc<str>), CanonicalSlot> = HashMap::new();

    #[allow(clippy::result_large_err)]
    while let Some(rule) = rules.pop_front() {
        if rule.operator == RuleOperator::Fallback {
            rules.push_front(rule);
            return canonical;
        }

        let Some(selector) = ruleseq_to_selector(&rule) else {
            rules.push_front(rule);
            return canonical;
        };

        let (selector, _) = selector.into_unpack_back();
        let state = (rule.kind, rule.comment);

        // First, unset all values of this selector for other rules
        canonical
            .iter_mut()
            .filter(|(other_state, _)| **other_state != state)
            .for_each(|(_, slot)| slot.update(&selector, |val| val.clear()));

        // Then, insert current time spans
        let entry = canonical.entry(state).or_default();

        if rule.operator == RuleOperator::Normal && rule.kind != RuleKind::Closed {
            // If the rule is not explicitly targeting a closed kind, then it
            // overrides previous rules for the whole day.
            entry.set(&selector, &rule.time_selector.time);
        } else {
            entry.update(&selector, |content| {
                content.extend_from_slice(rule.time_selector.time.as_slice())
            })
        }
    }

    canonical
}

pub(crate) fn canonical_to_seq2(mut canonical: Canonical2) -> Vec<RuleSequence> {
    // Order output rules from open to unknown
    canonical.remove(&(RuleKind::Closed, Arc::default()));
    let mut keys_ordered: Vec<_> = canonical.keys().cloned().collect();
    keys_ordered.sort();

    // Fill output
    let mut result = Vec::with_capacity(canonical.len());

    for state in keys_ordered {
        let Some(mut slot) = canonical.remove(&state) else {
            core::hint::cold_path(); // cannot happend
            continue;
        };

        while let Some((time, selector)) = slot.pop_filter(|spans| !spans.is_empty()) {
            let (kind, comment) = state.clone();

            // TODO: merge
            let time_selector = TimeSelector { time };

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

            result.push(RuleSequence {
                day_selector,
                time_selector,
                kind,
                operator: RuleOperator::Normal,
                comment,
            });
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
            time: MakeCanonical::into_selector(rgs_time, false),
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
