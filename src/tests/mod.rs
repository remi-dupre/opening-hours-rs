mod holiday_selector;
mod issues;
mod month_selector;
mod next_change;
mod parser;
mod regression;
mod rules;
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

/// Wraps input function but panics if it runs longer than specified timeout.
pub fn exec_with_timeout<R: Send + 'static>(
    timeout: Duration,
    f: impl FnOnce() -> R + Send + 'static,
) -> R {
    if cfg!(feature = "disable-test-timeouts") {
        return f();
    }

    let result = Arc::new(Mutex::new(None));
    let finished = Arc::new(Condvar::new());

    let _runner = {
        let f = black_box(f);
        let result = result.clone();
        let finished = finished.clone();

        thread::spawn(move || {
            let elapsed = {
                let start = Instant::now();
                let res = f();
                (start.elapsed(), res)
            };

            *result.lock().expect("failed to write result") = Some(elapsed);
            finished.notify_all();
        })
    };

    let (mut result, _) = finished
        .wait_timeout(result.lock().expect("failed to fetch result"), timeout)
        .expect("poisoned lock");

    let Some((elapsed, res)) = result.take() else {
        panic!("exec stopped due to {timeout:?} timeout");
    };

    if elapsed > timeout {
        panic!("exec ran for {elapsed:?}");
    }

    res
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
