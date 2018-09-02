use glob;
use harness::test::Test;
use serde_json::from_reader;

#[derive(Deserialize)]
struct Suite {
    #[serde(default)]
    pub tests: Vec<Test>,
}

pub fn get_tests() -> Vec<Test> {
    let mut tests = Vec::new();

    for file in read_tests!("html5lib-tests/tokenizer/*.test") {
        tests.extend(from_reader::<_, Suite>(file).unwrap().tests);
    }

    tests.iter_mut().for_each(|t| t.init());

    tests
}
