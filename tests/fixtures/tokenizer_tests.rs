use glob;
use harness::tokenizer_test::TokenizerTest;
use serde_json::from_reader;
use test::TestDescAndFn;

#[derive(Deserialize)]
struct Suite {
    #[serde(default)]
    pub tests: Vec<TokenizerTest>,
}

pub fn get_tests() -> Vec<TestDescAndFn> {
    let mut tests = Vec::new();

    for file in read_test_data!("html5lib-tests/tokenizer/*.test") {
        tests.extend(from_reader::<_, Suite>(file).unwrap().tests);
    }

    convert_tokenizer_tests!(tests)
}
