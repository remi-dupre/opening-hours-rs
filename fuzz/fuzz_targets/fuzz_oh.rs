#![no_main]
use arbitrary::Arbitrary;
use chrono::DateTime;
use libfuzzer_sys::{fuzz_target, Corpus};

use std::fmt::Debug;

use opening_hours::{Context, Coordinates, Localize, OpeningHours};

#[derive(Arbitrary, Clone)]
pub struct Data {
    date_secs: i64,
    date_nsecs: u32,
    oh: String,
    coords: Option<[f64; 2]>,
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Data");

        if let Some(date) = DateTime::from_timestamp(self.date_secs, self.date_nsecs) {
            debug.field("date", &date.naive_utc());
        }

        debug.field("oh", &self.oh);

        if let Some(coords) = &self.coords {
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

    let Some(date) = DateTime::from_timestamp(data.date_secs, data.date_nsecs) else {
        return Corpus::Reject;
    };

    let date = date.naive_utc();

    let Ok(oh_1) = data.oh.parse::<OpeningHours>() else {
        return Corpus::Reject;
    };

    let oh_2: OpeningHours = oh_1.to_string().parse().unwrap_or_else(|err| {
        eprintln!("[ERR] Initial Expression: {}", data.oh);
        eprintln!("[ERR] Invalid stringified Expression: {oh_1}");
        panic!("{err}")
    });

    if let Some([lat, lon]) = data.coords {
        let Some(coords) = Coordinates::new(lat, lon) else {
            return Corpus::Reject;
        };

        let ctx = Context::from_coords(coords);
        let date = ctx.locale.datetime(date);
        let oh_1 = oh_1.with_context(ctx.clone());
        let oh_2 = oh_2.with_context(ctx.clone());
        assert_eq!(oh_1.is_open(date), oh_2.is_open(date));
        assert_eq!(oh_1.next_change(date), oh_2.next_change(date));
    } else {
        assert_eq!(oh_1.is_open(date), oh_2.is_open(date));
        assert_eq!(oh_1.next_change(date), oh_2.next_change(date));
    }

    Corpus::Keep
});
