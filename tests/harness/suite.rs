use glob;
use serde_json;
use std::fs::File;
use std::io::BufReader;
use super::test_case::TestCase;

#[derive(Deserialize)]
struct Suite {
    #[serde(default)]
    pub tests: Vec<TestCase>,
}

macro_rules! read_tests {
    ($path:expr) => {
        glob::glob(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/", $path))
            .unwrap()
            .map(|path| BufReader::new(File::open(path.unwrap()).unwrap()))
    };
}

pub fn get_tests() -> Vec<TestCase> {
    let mut tests = Vec::new();

    for file in read_tests!("html5lib-tests/tokenizer/*.test") {
        tests.extend(serde_json::from_reader::<_, Suite>(file).unwrap().tests);
    }

    tests
}
