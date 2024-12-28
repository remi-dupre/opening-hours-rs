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

    let result = Arc::new(Mutex::new((None, false)));
    let finished = Arc::new(Condvar::new());

    struct DetectPanic<R>(Arc<Mutex<(Option<R>, bool)>>);

    impl<R> Drop for DetectPanic<R> {
        fn drop(&mut self) {
            if thread::panicking() {
                self.0.lock().expect("failed to write panic").1 = true;
            }
        }
    }

    let _runner = {
        let f = black_box(f);
        let result = result.clone();
        let finished = finished.clone();

        thread::spawn(move || {
            let _panic_guard = DetectPanic(result.clone());

            let elapsed = {
                let start = Instant::now();
                let res = f();
                (start.elapsed(), res)
            };

            result.lock().expect("failed to write result").0 = Some(elapsed);
            finished.notify_all();
        })
    };

    let (mut result, _) = finished
        .wait_timeout(result.lock().expect("failed to fetch result"), timeout)
        .expect("poisoned lock");

    if result.1 {
        panic!("exec stopped due to panic");
    }

    let Some((elapsed, res)) = result.0.take() else {
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
                use $crate::TzLocation;
                TzLocation::from_coords($coord.0, $coord.1)
            }))?;

        $expression
            .parse::<OpeningHours>()?
            .with_context(ctx)
            .schedule_at(date!($date))
    }};
}
