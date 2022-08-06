mod holiday_selector;
mod issues;
mod month_selector;
mod next_change;
mod next_change_hint;
mod parser;
mod rules;
mod selective;
mod time_selector;
mod week_selector;
mod weekday_selector;
mod year_selector;

use criterion::black_box;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn sample() -> impl Iterator<Item = &'static str> {
    include_str!("data/sample.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

fn exec_with_timeout(f: impl FnOnce() + Send + 'static, timeout: Duration) -> bool {
    let result = Arc::new(Mutex::new(None));
    let finished = Arc::new(Condvar::new());

    let _runner = {
        let f = black_box(f);
        let result = result.clone();
        let finished = finished.clone();

        thread::spawn(move || {
            let elapsed = {
                let start = Instant::now();
                f();
                start.elapsed()
            };

            *result.lock().expect("failed to write result") = Some(elapsed);
            finished.notify_all();
        })
    };

    let (result, _) = finished
        .wait_timeout(result.lock().expect("failed to fetch result"), timeout)
        .expect("poisoned lock");

    result.map(|elapsed| elapsed < timeout).unwrap_or(false)
}

#[macro_export]
macro_rules! assert_speed {
    ( $expr: expr ; $time: literal ms ) => {{
        use std::time::Duration;
        use $crate::tests::exec_with_timeout;

        assert!(exec_with_timeout(
            || {
                $expr;
            },
            Duration::from_millis(100),
        ));
    }};
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
}

#[macro_export]
macro_rules! schedule_at {
    ( $expression: expr, $date: expr ) => {{
        use $crate::{date, OpeningHours};
        OpeningHours::parse($expression)?.schedule_at(date!($date))
    }};
    ( $expression: expr, $date: expr, $region: expr ) => {{
        use $crate::{date, OpeningHours};

        OpeningHours::parse($expression)?
            .with_region($region)
            .schedule_at(date!($date))
    }};
}
