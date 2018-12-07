use crate::harness::tokenizer_test::TestCase;
use lazy_static::lazy_static;

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

mod feedback_tests;
mod html5lib_tests;

fn get_test_cases() -> Vec<TestCase> {
    let mut tests = Vec::new();

    tests.append(&mut self::html5lib_tests::get_test_cases());
    tests.append(&mut self::feedback_tests::get_test_cases());

    tests.iter_mut().for_each(|t| {
        t.init();

        if t.ignored {
            println!("Ignoring test: `{}`", t.description);
        }
    });

    tests
}

lazy_static! {
    pub static ref TEST_CASES: Vec<TestCase> = get_test_cases();
}
