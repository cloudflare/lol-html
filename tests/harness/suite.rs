use super::test::Test;
use glob;
use serde_json::from_reader;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize)]
struct Suite {
    #[serde(default)]
    pub tests: Vec<Test>,
}

macro_rules! read_tests {
    ($path:expr) => {
        glob::glob(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/", $path))
            .unwrap()
            .map(|path| BufReader::new(File::open(path.unwrap()).unwrap()))
    };
}

pub fn get_tests() -> Vec<Test> {
    let mut tests = Vec::new();

    for file in read_tests!("html5lib-tests/tokenizer/*.test") {
        tests.extend(from_reader::<_, Suite>(file).unwrap().tests);
    }

    tests.iter_mut().for_each(|t| t.init());

    tests
}
