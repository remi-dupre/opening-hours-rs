pub(crate) mod canonical;
pub(crate) mod frame;
pub(crate) mod paving;

use crate::normalize::paving::{EmptyPavingSelector, Paving, Paving4D};
use crate::rules::day::DaySelector;
use crate::rules::time::TimeSelector;
use crate::rules::{RuleOperator, RuleSequence};
use crate::RuleKind;

use self::canonical::{Canonical, CanonicalSelector, MakeCanonical};

// --
// -- Normalization Logic
// --

/// Convert a rule sequence to a n-dim selector.
pub(crate) fn ruleseq_to_selector(rs: &RuleSequence) -> Option<CanonicalSelector> {
    let ds = &rs.day_selector;

    let selector = EmptyPavingSelector
        .dim_front(MakeCanonical::try_from_iterator(&ds.weekday)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.week)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.monthday)?)
        .dim_front(MakeCanonical::try_from_iterator(&ds.year)?)
        .dim_front(MakeCanonical::try_from_iterator(&rs.time_selector.time)?);

    Some(selector)
}

/// Convert a canonical paving back into a rules sequence.
pub(crate) fn canonical_to_seq(mut canonical: Canonical) -> impl Iterator<Item = RuleSequence> {
    // Keep track of the days that have already been outputed. This allows to use an additional
    // rule if it is absolutly required only.
    let mut days_covered = Paving4D::default();

    std::iter::from_fn(move || {
        // Extract open periods first, then unknowns
        let ((kind, comments), selector) = [RuleKind::Open, RuleKind::Unknown, RuleKind::Closed]
            .into_iter()
            .find_map(|target_kind| {
                canonical.pop_filter(|(kind, comments)| {
                    *kind == target_kind
                        && (target_kind != RuleKind::Closed || !comments.is_empty())
                })
            })?;

        let (rgs_time, day_selector) = selector.into_unpack_front();

        let operator = {
            if days_covered.is_val(&day_selector, &false) {
                RuleOperator::Normal
            } else {
                RuleOperator::Additional
            }
        };

        days_covered.set(&day_selector, &true);
        let (rgs_year, day_selector) = day_selector.into_unpack_front();
        let (rgs_monthday, day_selector) = day_selector.into_unpack_front();
        let (rgs_week, day_selector) = day_selector.into_unpack_front();
        let (rgs_weekday, _) = day_selector.into_unpack_front();

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
            comments,
        })
    })
}
