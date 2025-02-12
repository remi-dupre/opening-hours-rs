#![no_main]
use libfuzzer_sys::{fuzz_target, Corpus};
use opening_hours::fuzzing::{run_fuzz_oh, Data};

fuzz_target!(|data: Data| -> Corpus {
    if run_fuzz_oh(data) {
        Corpus::Keep
    } else {
        Corpus::Reject
    }
});
