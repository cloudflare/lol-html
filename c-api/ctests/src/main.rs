//! The test runner for the C API tests.

extern "C" {
    fn run_tests() -> usize;
}

fn main() {
    unsafe {
        run_tests();
    }
}
