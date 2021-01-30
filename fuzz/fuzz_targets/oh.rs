#![no_main]
use chrono::NaiveDateTime;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut data = std::str::from_utf8(data).unwrap_or("").splitn(2, '\n');
    let raw_date = data.next().unwrap_or_default();
    let raw_oh = data.next().unwrap_or_default();

    let date = NaiveDateTime::parse_from_str(raw_date, "%d-%m-%Y %H:%M:%S");
    let oh = opening_hours::parse(raw_oh);

    if let (Ok(date), Ok(oh)) = (date, oh) {
        eprintln!("oh: '{}' -- date: '{}'", raw_oh, raw_date);
        let _ = oh.is_open(date);
        let _ = oh.next_change(date);
    }
});
