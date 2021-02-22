#![no_main]
use arbitrary::Arbitrary;
use chrono::NaiveDateTime;
use libfuzzer_sys::fuzz_target;
use opening_hours::OpeningHours;

#[derive(Arbitrary, Clone, Debug)]
pub struct Data {
    date_secs: i64,
    date_nsecs: u32,
    oh: String,
}

fuzz_target!(|data: Data| {
    if let Some(date) = NaiveDateTime::from_timestamp_opt(data.date_secs, data.date_nsecs) {
        if let Ok(oh) = OpeningHours::parse(&data.oh) {
            eprintln!(
                "oh: {:?} -- date: {}",
                data.oh,
                date.format("%Y-%m-%d %H:%M:%S")
            );

            let _ = oh.is_open(date);
            let _ = oh.next_change(date);
        }
    }
});
