#![no_main]
use arbitrary::Arbitrary;
use chrono::DateTime;
use libfuzzer_sys::{fuzz_target, Corpus};
use opening_hours::opening_hours::REGION_HOLIDAYS;

use std::fmt::Debug;

use opening_hours::OpeningHours;

#[derive(Arbitrary, Clone)]
pub struct Data {
    date_secs: i64,
    date_nsecs: u32,
    oh: String,
    region: String,
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Data");

        if let Some(date) = DateTime::from_timestamp(self.date_secs, self.date_nsecs) {
            debug.field("date", &date.naive_utc());
        }

        debug.field("oh", &self.oh);
        debug.field("region", &self.region);
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

    let Ok(mut oh_1) = OpeningHours::parse(&data.oh) else {
        return Corpus::Reject;
    };

    let mut oh_2 = OpeningHours::parse(&oh_1.to_string()).unwrap_or_else(|err| {
        eprintln!("[ERR] Initial Expression: {}", data.oh);
        eprintln!("[ERR] Invalid stringified Expression: {oh_1}");
        panic!("{err}")
    });

    if data.region.is_empty() {
        if !REGION_HOLIDAYS.contains_key(data.region.as_str()) {
            return Corpus::Reject;
        }

        oh_1 = oh_1.with_region(&data.region);
        oh_2 = oh_2.with_region(&data.region);
    }

    assert_eq!(oh_1.is_open(date), oh_2.is_open(date));
    assert_eq!(oh_1.next_change(date), oh_2.next_change(date));
    Corpus::Keep
});
