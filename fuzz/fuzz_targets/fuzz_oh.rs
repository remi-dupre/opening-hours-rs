#![no_main]
use fuzz::{run_fuzz_oh, Data};
use libfuzzer_sys::{fuzz_target, Corpus};

fuzz_target!(|data: Data| -> Corpus {
    if run_fuzz_oh(data) {
        Corpus::Keep
    } else {
        Corpus::Reject
    }
});
