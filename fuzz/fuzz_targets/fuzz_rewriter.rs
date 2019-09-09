#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate test_case;

use test_case::run_rewriter;

fuzz_target!(|data: &[u8]| {
    run_rewriter(data);
});
