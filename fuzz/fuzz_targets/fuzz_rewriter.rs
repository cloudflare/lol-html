#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate common;

use common::run_rewriter;

fuzz_target!(|data: &[u8]| {
    run_rewriter(data);
});
