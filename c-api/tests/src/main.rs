//! The test runner for the C API tests.

// ensure it's linked
use lolhtml as _;

extern "C" {
    fn run_tests() -> i32;
}

fn main() {
    unsafe { std::process::exit(run_tests()) }
}
