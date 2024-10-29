#![no_main]
use arbitrary::Arbitrary;
use chrono::DateTime;
use libfuzzer_sys::fuzz_target;
use opening_hours::OpeningHours;

#[derive(Arbitrary, Clone, Debug)]
pub struct Data {
    date_secs: i64,
    date_nsecs: u32,
    oh: String,
}

fuzz_target!(|data: Data| {
    if data.oh.contains('=') {
        // The fuzzer spends way too much time building comments.
        return;
    }

    if let Some(date) = DateTime::from_timestamp(data.date_secs, data.date_nsecs) {
        let date = date.naive_utc();

        if let Ok(oh) = OpeningHours::parse(&data.oh) {
            let _ = oh.is_open(date);
            let _ = oh.next_change(date);
        }
    }
});
