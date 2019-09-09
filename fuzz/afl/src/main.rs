#[macro_use]
extern crate afl;
extern crate test_case;

use test_case::run_rewriter;

fn main() {
    fuzz!(|data: &[u8]| {
      run_rewriter(data);
    });
}
