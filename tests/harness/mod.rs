#[macro_use]
pub mod functional_testing;

macro_rules! create_test {
    ($name:expr, $body:tt) => {{
        use test::{ShouldPanic, TestDesc, TestDescAndFn, TestFn, TestName};

        TestDescAndFn {
            desc: TestDesc {
                name: TestName::DynTestName($name),
                ignore: false,
                should_panic: ShouldPanic::No,
                allow_fail: false,
            },
            testfn: TestFn::DynTestFn(Box::new(move || $body)),
        }
    }};
}

macro_rules! test_fixture {
    ($fixture_name:expr, { $(test($name:expr, $body:tt);)+}) => {
        use test::TestDescAndFn;
        use std::fmt::Write;

        pub fn get_tests() -> Vec<TestDescAndFn> {
            let mut tests = Vec::new();

            $({
                let mut name = String::new();

                write!(&mut name, "{} - {}", $fixture_name, $name).unwrap();

                tests.push(create_test!(name, $body));
            })+

            tests
        }
    };
}
