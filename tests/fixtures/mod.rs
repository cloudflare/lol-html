mod parser_feedback_tests;
mod tokenizer_tests;

use super::harness::test::Test;

pub fn get_tests() -> Vec<Test> {
    let mut tests = Vec::new();

    tests.extend(tokenizer_tests::get_tests());
    tests.extend(parser_feedback_tests::get_tests());

    tests
}
