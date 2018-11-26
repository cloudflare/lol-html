#[macro_use]
pub mod tokenizer_test;

macro_rules! read_test_data {
    ($path:expr) => {{
        use std::fs::File;
        use std::io::BufReader;

        glob::glob(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/", $path))
            .unwrap()
            .map(|path| BufReader::new(File::open(path.unwrap()).unwrap()))
            .collect::<Vec<BufReader<File>>>()
    }};
}

macro_rules! create_test {
    ($name:expr, $ignored:expr, $body:tt) => {{
        use test::{ShouldPanic, TestDesc, TestDescAndFn, TestFn, TestName};

        TestDescAndFn {
            desc: TestDesc {
                name: TestName::DynTestName($name),
                ignore: $ignored,
                should_panic: ShouldPanic::No,
                allow_fail: false,
            },
            testfn: TestFn::DynTestFn(Box::new(move || $body)),
        }
    }};
}

macro_rules! test_fixture {
    ($fixture_name:expr, { $(test($name:expr, $body:tt);)+}) => (
        use test::TestDescAndFn;
        use std::fmt::Write;

        pub fn get_tests() -> Vec<TestDescAndFn> {
            let mut tests = Vec::new();

            $({
                let mut name = String::new();

                write!(&mut name, "{} - {}", $fixture_name, $name).unwrap();

                tests.push(create_test!(name, false, $body));
            })+

            tests
        }
    );
}
