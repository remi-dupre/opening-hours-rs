#![no_main]
use arbitrary::Arbitrary;
use chrono::{DateTime, Datelike};
use libfuzzer_sys::{fuzz_target, Corpus};

use std::fmt::Debug;

use opening_hours::localization::{Coordinates, Localize};
use opening_hours::{Context, OpeningHours};

#[derive(Arbitrary, Clone, Debug)]
pub enum CompareWith {
    Stringified,
    Normalized,
}

#[derive(Arbitrary, Clone, Debug)]
pub enum Operation {
    DoubleNormalize,
    Compare(CompareWith),
}

#[derive(Arbitrary, Clone)]
pub struct Data {
    date_secs: i64,
    oh: String,
    coords: Option<[i16; 2]>,
    operation: Operation,
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

fuzz_target!(|data: Data| -> Corpus {
    if data.oh.contains('=') {
        // The fuzzer spends way too much time building comments.
        return Corpus::Reject;
    }

    let Some(date) = DateTime::from_timestamp(data.date_secs, 0) else {
        return Corpus::Reject;
    };

    let date = date.naive_utc();

    if date.year() < 1900 || date.year() > 9999 {
        return Corpus::Reject;
    }

    let Ok(oh_1) = data.oh.parse::<OpeningHours>() else {
        return Corpus::Reject;
    };

    match &data.operation {
        Operation::DoubleNormalize => {
            let normalized = oh_1.normalize();
            assert_eq!(normalized, normalized.clone().normalize());
        }
        Operation::Compare(compare_with) => {
            let oh_2: OpeningHours = match compare_with {
                CompareWith::Stringified => oh_1.to_string().parse().unwrap_or_else(|err| {
                    eprintln!("[ERR] Initial Expression: {}", data.oh);
                    eprintln!("[ERR] Invalid stringified Expression: {oh_1}");
                    panic!("{err}")
                }),
                CompareWith::Normalized => oh_1.clone().normalize(),
            };

            if let Some([lat, lon]) = data.coords_float() {
                let ctx = Context::from_coords(Coordinates::new(lat, lon).unwrap());
                let date = ctx.locale.datetime(date);
                let oh_1 = oh_1.clone().with_context(ctx.clone());
                let oh_2 = oh_2.with_context(ctx.clone());
                assert_eq!(oh_1.is_open(date), oh_2.is_open(date));
                assert_eq!(oh_1.next_change(date), oh_2.next_change(date));
            } else {
                assert_eq!(oh_1.is_open(date), oh_2.is_open(date));
                assert_eq!(oh_1.next_change(date), oh_2.next_change(date));
            }
        }
    }

    Corpus::Keep
});
