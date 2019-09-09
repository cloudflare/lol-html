#[macro_use] extern crate honggfuzz;
extern crate test_case;

use test_case::run_rewriter;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            run_rewriter(data);
        });
    }
}
