mod month_selector;
mod next_change;
mod parser;
mod rules;
mod selective;
mod time_selector;
mod week_selector;
mod weekday_selector;
mod year_selector;

#[macro_export]
macro_rules! date {
    ( $date:expr ) => {{
        use chrono::NaiveDate;
        NaiveDate::parse_from_str($date, "%Y-%m-%d").expect("invalid date literal")
    }};
}

#[macro_export]
macro_rules! datetime {
    ( $date:expr ) => {{
        use chrono::NaiveDateTime;
        NaiveDateTime::parse_from_str($date, "%Y-%m-%d %H:%M").expect("invalid datetime literal")
    }};
}

#[macro_export]
macro_rules! schedule_at {
    ( $expression:expr, $date:expr ) => {{
        use crate::date;
        use crate::parser::parse;

        parse($expression)?.schedule_at(date!($date))
    }};
}
