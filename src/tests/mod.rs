mod month_selector;
mod rules;
mod time_selector;
mod weekday_selector;

#[macro_export]
macro_rules! schedule_at {
    ( $expression:expr, $date:expr ) => {{
        use crate::parser::parse;
        use chrono::NaiveDate;

        parse($expression)?
            .schedule_at(NaiveDate::parse_from_str($date, "%Y-%m-%d").expect("invalid date"))
    }};
}
