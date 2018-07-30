extern crate cool_thing;
extern crate glob;
extern crate html5ever;
extern crate serde;
extern crate serde_json;

// From 'rustc-test' crate.
// Mirrors Rust's internal 'libtest'.
// https://doc.rust-lang.org/1.1.0/test/index.html
extern crate test;

#[macro_use]
extern crate serde_derive;

mod decoder;
mod suite;
mod test_case;
mod token;
mod unescape;

use suite::get_tests;
use test::{test_main, ShouldPanic, TestDesc, TestDescAndFn, TestFn, TestName};

fn main() {
    let args: Vec<_> = ::std::env::args().collect();

    let tests = get_tests()
        .into_iter()
        .map(|mut test| {
            test.init();

            TestDescAndFn {
                desc: TestDesc {
                    name: TestName::DynTestName(test.description.to_owned()),
                    ignore: test.ignored,
                    should_panic: ShouldPanic::No,
                    allow_fail: false,
                },
                testfn: TestFn::DynTestFn(Box::new(move || {
                    test.run();
                })),
            }
        })
        .collect();

    test_main(&args, tests);
}
