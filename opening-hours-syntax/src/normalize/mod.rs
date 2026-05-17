pub(crate) mod canonical;
pub(crate) mod frame;
pub(crate) mod paving;

use crate::normalize::paving::{EmptyPavingSelector, Paving, Paving4D, UnpackFromBack};
use crate::rules::day::DaySelector;
use crate::rules::time::TimeSelector;
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

/// Convert a canonical paving back into a rules sequence.
pub(crate) fn canonical_to_seq(canonical: Canonical) -> impl Iterator<Item = RuleSequence> {
    // Keep track of the days that have already been outputed. This allows to
    // use an additional rule only when necessary.
    let mut days_covered = Paving4D::default();

    // Parts of this paving will be removed until it is empty
    let mut canonical_remaining = canonical.clone();

    std::iter::from_fn(move || {
        // Extract open periods first, then unknowns
        let ((kind, comments), mut selector) =
            [RuleKind::Open, RuleKind::Unknown, RuleKind::Closed]
                .into_iter()
                .find_map(|target_kind| {
                    canonical_remaining.pop_filter(|(kind, comments)| {
                        *kind == target_kind
                            && (target_kind != RuleKind::Closed || !comments.is_empty())
                    })
                })?;

        // Merge consecutive intervals as much as possible if the hole between
        // two consecutive intervals was covered with the same value during a
        // previous extraction.
        selector.fill_holes({
            let canonical = &canonical;
            let val = (kind, comments.clone());
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
            comments,
        })
    })
}
