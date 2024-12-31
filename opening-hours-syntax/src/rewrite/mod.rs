use crate::extended_time::ExtendedTime;
use crate::rules::day::DaySelector;
use crate::rules::time::{Time, TimeSelector};
use crate::rules::{OpeningHoursExpression, RuleOperator};

fn day_is_subset(_small: &DaySelector, large: &DaySelector) -> bool {
    large.is_empty()
}

fn time_is_subset(_small: &TimeSelector, large: &TimeSelector) -> bool {
    large.time.iter().any(|span| {
        let Time::Fixed(start) = span.range.start else {
            return false;
        };

        let Time::Fixed(end) = span.range.end else {
            return false;
        };

        start == ExtendedTime::new(0, 0).unwrap() && end >= ExtendedTime::new(23, 59).unwrap()
    })
}

pub(crate) fn simplify_expression(mut expr: OpeningHoursExpression) -> OpeningHoursExpression {
    let mut i = 1;

    while i < expr.rules.len() {
        if expr.rules[i - 1].operator != RuleOperator::Normal
            || expr.rules[i].operator != RuleOperator::Normal
        {
            continue;
        }

        if day_is_subset(&expr.rules[i - 1].day_selector, &expr.rules[i].day_selector)
            && time_is_subset(
                &expr.rules[i - 1].time_selector,
                &expr.rules[i].time_selector,
            )
        {
            expr.rules.remove(i - 1);
            continue;
        }

        if day_is_subset(&expr.rules[i].day_selector, &expr.rules[i - 1].day_selector)
            && time_is_subset(
                &expr.rules[i].time_selector,
                &expr.rules[i - 1].time_selector,
            )
        {
            expr.rules.remove(i);
            continue;
        }

        i += 1;
    }

    expr
}
