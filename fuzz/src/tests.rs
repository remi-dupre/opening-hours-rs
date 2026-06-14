use std::fs::File;
use std::io::Read;
use std::path::Path;

use arbitrary::{Arbitrary, Unstructured};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::{run_fuzz_oh, CompareWith, Data, Operation};

#[test]
fn no_fuzz_before_1900() {
    let date_secs = {
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1899, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        )
        .and_utc()
        .timestamp()
    };

    let data = Data {
        date_secs,
        oh: "24/7".to_string(),
        coords: None,
        operation: Operation::Compare(CompareWith::Normalized),
    };

    assert!(!run_fuzz_oh(data));
}

#[test]
fn no_fuzz_after_9999() {
    let date_secs = {
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(10_000, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        )
        .and_utc()
        .timestamp()
    };

    let data = Data {
        date_secs,
        oh: "24/7".to_string(),
        coords: None,
        operation: Operation::Compare(CompareWith::Normalized),
    };

    assert!(!run_fuzz_oh(data));
}

#[test]
fn no_fuzz_with_comments() {
    let data = Data {
        date_secs: 0,
        oh: "24/7 = Some comment".to_string(),
        coords: None,
        operation: Operation::Compare(CompareWith::Normalized),
    };

    assert!(!run_fuzz_oh(data));
}

#[test]
fn no_fuzz_invalid_expression() {
    let data = Data {
        date_secs: 0,
        oh: "[invalid expression]".to_string(),
        coords: None,
        operation: Operation::Compare(CompareWith::Normalized),
    };

    assert!(!run_fuzz_oh(data));
}

#[test]
fn fuzz_oh() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join("fuzz_oh");

    let dir = std::fs::read_dir(path).expect("could not open corpus directory");

    for entry in dir {
        let entry = entry.expect("failed to iter corpus directory");
        eprintln!("Running fuzz corpus file {:?}", entry.path());
        let mut file = File::open(entry.path()).expect("failed to open corpus example");
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).expect("failed to read file");
        let data = Data::arbitrary(&mut Unstructured::new(&bytes)).expect("could not parse corpus");
        eprintln!("Input: {data:?}");
        let should_be_in_corpus = run_fuzz_oh(data);
        eprintln!("Output: {should_be_in_corpus:?}");
    }
}
