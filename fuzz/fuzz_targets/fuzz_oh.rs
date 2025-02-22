#![no_main]
use fuzz::{Data, run_fuzz_oh};
use libfuzzer_sys::{Corpus, fuzz_target};

fuzz_target!(|data: Data| -> Corpus {
    if run_fuzz_oh(data) {
        Corpus::Keep
    } else {
        Corpus::Reject
    }
});
