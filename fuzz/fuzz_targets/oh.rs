#![no_main]
use arbitrary::Arbitrary;
use chrono::DateTime;
use libfuzzer_sys::fuzz_target;

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

fuzz_target!(|data: Data| {
    if data.oh.contains('=') {
        // The fuzzer spends way too much time building comments.
        return;
    }

    let Some(date) = DateTime::from_timestamp(data.date_secs, data.date_nsecs) else {
        return;
    };

    let date = date.naive_utc();

    let Ok(oh) = OpeningHours::parse(&data.oh) else {
        return;
    };

    let oh = oh.with_region(&data.region);
    let _ = oh.is_open(date);
    let _ = oh.next_change(date);
});
