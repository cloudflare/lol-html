use encoding_rs::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref TEST_CRITICAL_SECTION_MUTEX: Mutex<()> = Mutex::new(());
}

macro_rules! ignore {
    (@info $($args:expr),+) => {
        if std::env::var("IGNORES_VERBOSE").is_ok() {
            println!($($args),+);
        }
    };

    (@total $type:expr, $count:expr) => {
        println!("Ignoring {} {} tests, run with `IGNORES_VERBOSE=1` to get more info.", $count, $type);
    };
}

mod io;

#[macro_use]
pub mod functional_testing;

#[macro_use]
mod parse;

pub use self::io::{Input, Output};
pub use self::parse::{parse, TestTransformController};

pub static ASCII_COMPATIBLE_ENCODINGS: [&Encoding; 36] = [
    BIG5,
    EUC_JP,
    EUC_KR,
    GB18030,
    GBK,
    IBM866,
    ISO_8859_2,
    ISO_8859_3,
    ISO_8859_4,
    ISO_8859_5,
    ISO_8859_6,
    ISO_8859_7,
    ISO_8859_8,
    ISO_8859_8_I,
    ISO_8859_10,
    ISO_8859_13,
    ISO_8859_14,
    ISO_8859_15,
    ISO_8859_16,
    KOI8_R,
    KOI8_U,
    MACINTOSH,
    SHIFT_JIS,
    UTF_8,
    WINDOWS_874,
    WINDOWS_1250,
    WINDOWS_1251,
    WINDOWS_1252,
    WINDOWS_1253,
    WINDOWS_1254,
    WINDOWS_1255,
    WINDOWS_1256,
    WINDOWS_1257,
    WINDOWS_1258,
    X_MAC_CYRILLIC,
    X_USER_DEFINED,
];

macro_rules! create_test {
    ($name:expr, $should_panic:expr, $body:tt) => {{
        use test::{TestDesc, TestDescAndFn, TestFn, TestName};

        TestDescAndFn {
            desc: TestDesc {
                name: TestName::DynTestName($name),
                ignore: false,
                should_panic: $should_panic,
                allow_fail: false,
            },
            testfn: TestFn::DynTestFn(Box::new(move || $body)),
        }
    }};
}

macro_rules! test_fixture {
    ($fixture_name:expr, { $($test_defs:tt)+}) => {
        use test::{ShouldPanic, TestDescAndFn};
        use std::fmt::Write;

        pub fn get_tests() -> Vec<TestDescAndFn> {
            let mut tests = Vec::default();

            test_fixture!(@test |$fixture_name, tests|> $($test_defs)+);

            tests
        }
    };

    // NOTE: recursively expand all tests
    (@test |$fixture_name:expr, $tests:ident|>
        test($name:expr, expect_panic:$panic_msg:expr, $body:tt);
        $($rest:tt)*
    ) => {
        test_fixture!(@test_body
            $fixture_name,
            $tests,
            $name,
            {
                use std::panic;

                // NOTE: since we generate tests dynamically we need to use a rip-off
                // of Rust's test harness which is for some reason leaks panic error
                // messages to the output even if they are expected.
                //
                // To workaround that, we deploy noop panic handler before the code
                // that should panic and later restore the original handler to get
                // proper assertion error reporting for the rest of the tests.
                //
                // Test harness uses multiple threads to run tests and the panic hook
                // is global to the whole process, so we need to use mutex to synchronise
                // access to it.
                let res = {
                    let mut cs = crate::harness::TEST_CRITICAL_SECTION_MUTEX.lock().unwrap();
                    let original_hook = panic::take_hook();

                    panic::set_hook(Box::new(|_|{}));

                    let res = panic::catch_unwind(|| { $body });

                    panic::set_hook(original_hook);

                    // NOTE: mutex should be used, otherwise it unlocks immediately.
                    *cs = ();

                    res
                };

                let panic_err = res.expect_err("Panic expected");
                let msg = panic_err.downcast_ref::<&str>().unwrap();

                assert_eq!(msg, &$panic_msg);
            }
        );

        test_fixture!(@test |$fixture_name, $tests|> $($rest)*);
    };

    (@test |$fixture_name:expr, $tests:ident|>
        test($name:expr, $body:tt);
        $($rest:tt)*
    ) => {
        test_fixture!(@test_body $fixture_name, $tests, $name, $body);
        test_fixture!(@test |$fixture_name, $tests|> $($rest)*);
    };


    // NOTE: end of recursion
    (@test |$fixture_name:expr, $tests:ident|>) => {};

    (@test_body $fixture_name:expr, $tests:ident, $name:expr, $body:tt) => {{
        let mut name = String::new();

        write!(&mut name, "{} - {}", $fixture_name, $name).unwrap();
        $tests.push(create_test!(name, ShouldPanic::No, $body));
    }};
}

macro_rules! test_modules {
    ($($m:ident),+) => {
        $(mod $m;)+

        use test::TestDescAndFn;

        pub fn get_tests() -> Vec<TestDescAndFn> {
            let mut tests = Vec::default();

            $(tests.extend($m::get_tests());)+

            tests
        }
    };
}
