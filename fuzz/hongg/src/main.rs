#[macro_use] extern crate honggfuzz;
extern crate common;

use common::run_rewriter;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            run_rewriter(data);
        });
    }
}
