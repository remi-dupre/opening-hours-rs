use core::ops::Range;
use std::sync::Arc;

use crate::normalize::paving::{EmptyPavingSelector, Paving, Paving1D};
use crate::rules::time::{Time, TimeSelector, TimeSpan};
use crate::{ExtendedTime, RuleKind};

/// For a given day, the list of rules to apply from left to right.
pub(crate) type TimeRules = Vec<((RuleKind, Arc<str>), TimeSelector)>;

// TODO: doc
pub(crate) fn normalize_time_rules(slot: TimeRules) -> TimeRules {
    let mut result = Vec::new();
    let mut canonical = TimeSelectorPaving::default();

    // Iter over each spans of the time selectors with intention to individualy push them to the
    // canonical time selector.
    let mut spans = (slot.into_iter())
        .flat_map(|(state, selector)| {
            selector
                .spans
                .into_iter()
                .map(move |span| (state.clone(), span))
        })
        .peekable();

    while let Some((state, span)) = spans.next() {
        // If the span can be turned into a simple range, then we can just add it to the canonical
        // structure.
        if let Some(range) = time_span_to_daily_ranges(&span) {
            canonical.set_time_range(range, state);
            continue;
        }

        // If the span can't be converted to a simple range, we need to stop iterating.
        let mut non_canonical = TimeSelector { spans: vec![span] };

        // However, _can_ continue iterating as long as the state doesn't change because the order
        // doesn't matter in this case.
        while let Some((_, extra_span)) = spans.next_if(|(extra_state, _)| *extra_state == state) {
            if let Some(range) = time_span_to_daily_ranges(&extra_span) {
                canonical.set_time_range(range, state.clone());
            } else {
                non_canonical.spans.push(extra_span)
            }
        }

        // Add selectors infered from the canonical representation and clear the struct.
        result.extend(canonical.into_time_selector());
        canonical = TimeSelectorPaving::default();

        // Sort the non canonical spans to get a deterministic result and add them after the
        // canonical selectors.
        non_canonical.spans.sort();
        result.push((state, non_canonical));
    }

    result.extend(canonical.into_time_selector());
    result
}

// TODO: doc
fn time_span_to_daily_ranges(span: &TimeSpan) -> Option<Range<ExtendedTime>> {
    if span.open_end || span.repeats.is_some() {
        return None;
    }

    match (span.range.start, span.range.end) {
        (Time::Fixed(start), Time::Fixed(end))
            if start <= end && end <= ExtendedTime::MIDNIGHT_24 =>
        {
            Some(start..end)
        }
        _ => None,
    }
}

/// A canonical time selector is an increasing sequence of time ranges.
///
/// TODO: test overlapping
type TimeSelectorPaving = Paving1D<ExtendedTime, (RuleKind, Arc<str>)>;

impl TimeSelectorPaving {
    fn set_time_range(&mut self, range: Range<ExtendedTime>, state: (RuleKind, Arc<str>)) {
        let selector = EmptyPavingSelector.dim_front(vec![range]);
        self.set(&selector, &state);
    }

    /// TODO: doc
    fn into_time_selector(mut self) -> TimeRules {
        let mut result = Vec::new();

        #[allow(clippy::type_complexity)]
        let pop_priority_order: [Box<dyn Fn(&(RuleKind, Arc<str>)) -> bool>; _] = [
            // 1. Open spans WITHOUT comment
            Box::new(|(kind, comment)| *kind == RuleKind::Open && comment.is_empty()),
            // 2. Open spans WITH comment
            Box::new(|(kind, _)| *kind == RuleKind::Open),
            // 3. Unknown spans WITHOUT comment
            Box::new(|(kind, comment)| *kind == RuleKind::Unknown && comment.is_empty()),
            // 4. Unknown spans WITH comment
            Box::new(|(kind, _)| *kind == RuleKind::Unknown),
            // 5. Closed spans WITH comment
            Box::new(|(_, comment)| !comment.is_empty()),
        ];

        for pop_priority in pop_priority_order {
            while let Some((state, selector)) = self.pop_filter(&pop_priority) {
                let (ranges, _) = selector.into_unpack_front();

                let spans = ranges
                    .into_iter()
                    .map(|range| TimeSpan {
                        range: Time::Fixed(range.start)..Time::Fixed(range.end),
                        open_end: false,
                        repeats: None,
                    })
                    .collect();

                result.push((state, TimeSelector { spans }));
            }
        }

        result
    }
}
