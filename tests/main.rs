extern crate cool_thing;
extern crate glob;
extern crate serde;
extern crate serde_json;

// From 'rustc-test' crate.
// Mirrors Rust's internal 'libtest'.
// https://doc.rust-lang.org/1.1.0/test/index.html
extern crate test;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate itertools;

#[macro_use]
extern crate html5ever;

#[macro_use]
mod harness;
mod fixtures;

use fixtures::get_tests;
use test::test_main;

fn main() {
    let args: Vec<_> = ::std::env::args().collect();

    test_main(&args, get_tests());
}
