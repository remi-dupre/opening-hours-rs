pub(crate) mod stats;

mod country;
mod holiday_selector;
mod issues;
mod localization;
mod month_selector;
mod next_change;
mod parser;
mod regression;
mod rules;
mod schedule;
mod time_selector;
mod week_selector;
mod weekday_selector;
mod year_selector;

fn sample() -> impl Iterator<Item = &'static str> {
    include_str!("data/sample.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

#[macro_export]
macro_rules! date {
    ( $date: expr ) => {{
        use chrono::NaiveDate;
        NaiveDate::parse_from_str($date, "%Y-%m-%d").expect("invalid date literal")
    }};
}

#[macro_export]
macro_rules! datetime {
    ( $date: expr ) => {{
        use chrono::NaiveDateTime;
        NaiveDateTime::parse_from_str($date, "%Y-%m-%d %H:%M").expect("invalid datetime literal")
    }};
    ( $date: expr, $tz: expr ) => {{
        use chrono::TimeZone;

        $tz.from_local_datetime(&datetime!($date))
            .single()
            .expect("ambiguous input datetime")
    }};
}

#[macro_export]
macro_rules! schedule_at {
    (
        $expression: expr,
        $date: expr
        $( , region = $region: expr )?
        $( , coord = $coord: expr )?
        $( , )?
    ) => {{
        use $crate::{date, Context, OpeningHours};

        let ctx = Context::default()
            $( .with_holidays($region.holidays()) )?
            $( .with_locale({
                use $crate::localization::{Coordinates, TzLocation};
                let coords = Coordinates::new($coord.0, $coord.1).unwrap();
                TzLocation::from_coords(coords)
            }))?;

        $expression
            .parse::<OpeningHours>()?
            .with_context(ctx)
            .schedule_at(date!($date))
    }};
}
