//! Development module that shares the fuzzing logic between unit tests and
//! the actual fuzzing.
#[cfg(test)]
mod tests;

use arbitrary::Arbitrary;
use chrono::{DateTime, Datelike};

use std::fmt::Debug;

use opening_hours::localization::{Coordinates, Localize};
use opening_hours::{Context, OpeningHours};

const MAX_INTERVAL_RANGE: chrono::TimeDelta = chrono::TimeDelta::days(366 * 10);

/// A fuzzing example
#[derive(Arbitrary, Clone)]
pub struct Data {
    pub date_secs: i64,
    pub oh: String,
    pub coords: Option<[i16; 2]>,
    pub operation: Operation,
}

/// What operation to perform on the input
#[derive(Arbitrary, Clone, Debug)]
pub enum Operation {
    DoubleNormalize,
    Compare(CompareWith),
}

/// What transformation to apply on the input before comparing
#[derive(Arbitrary, Clone, Debug)]
pub enum CompareWith {
    Stringified,
    Normalized,
}

impl Data {
    fn coords_float(&self) -> Option<[f64; 2]> {
        self.coords.map(|coords| {
            [
                90.0 * coords[0] as f64 / (i16::MAX as f64 + 1.0),
                180.0 * coords[1] as f64 / (i16::MAX as f64 + 1.0),
            ]
        })
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Data");

        if let Some(date) = DateTime::from_timestamp(self.date_secs, 0) {
            debug.field("date", &date.naive_utc());
        }

        debug.field("operation", &self.operation);
        debug.field("oh", &self.oh);

        if let Some(coords) = &self.coords_float() {
            debug.field("coords", coords);
        }

        debug.finish()
    }
}

/// Run a fuzzing test and return `true` if the example should be kept in
/// corpus.
pub fn run_fuzz_oh(data: Data) -> bool {
    if data.oh.contains('=') {
        // The fuzzer spends way too much time building comments.
        return false;
    }

    let Some(date) = DateTime::from_timestamp(data.date_secs, 0) else {
        return false;
    };

    let date = date.naive_utc();

    if date.year() < 1900 || date.year() > 9999 {
        return false;
    }

    let Ok(oh_1) = data.oh.parse::<OpeningHours>() else {
        return false;
    };

    match &data.operation {
        Operation::DoubleNormalize => {
            let normalized = oh_1.normalize();
            assert_eq!(normalized, normalized.clone().normalize());
        }
        Operation::Compare(compare_with) => {
            let oh_2 = match compare_with {
                CompareWith::Normalized => oh_1.normalize(),
                CompareWith::Stringified => oh_1.to_string().parse().unwrap_or_else(|err| {
                    eprintln!("[ERR] Initial Expression: {}", data.oh);
                    eprintln!("[ERR] Invalid stringified Expression: {oh_1}");
                    panic!("{err}")
                }),
            };

            if let Some([lat, lon]) = data.coords_float() {
                let ctx = Context::from_coords(Coordinates::new(lat, lon).unwrap())
                    .approx_bound_interval_size(MAX_INTERVAL_RANGE);

                let date = ctx.locale.datetime(date);
                let oh_1 = oh_1.with_context(ctx.clone());
                let oh_2 = oh_2.with_context(ctx.clone());

                assert_eq!(oh_1.state(date), oh_2.state(date));
                assert_eq!(oh_1.next_change(date), oh_2.next_change(date));
            } else {
                let ctx = Context::default().approx_bound_interval_size(MAX_INTERVAL_RANGE);
                let oh_1 = oh_1.with_context(ctx.clone());
                let oh_2 = oh_2.with_context(ctx.clone());
                assert_eq!(oh_1.state(date), oh_2.state(date));
                assert_eq!(oh_1.next_change(date), oh_2.next_change(date));
            }
        }
    }

    true
}
