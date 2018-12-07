#[macro_use]
extern crate serde_derive;

#[macro_use]
mod harness;

mod fixtures;

use self::fixtures::get_tests;
use test::test_main;

fn main() {
    let args: Vec<_> = ::std::env::args().collect();

    test_main(&args, get_tests());
}
