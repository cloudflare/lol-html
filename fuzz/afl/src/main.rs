#[macro_use]
extern crate afl;
extern crate common;

use common::run_rewriter;

fn main() {
    fuzz!(|data: &[u8]| {
      run_rewriter(data);
    });
}
