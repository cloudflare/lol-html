mod tag_name_hash_tests;
mod tokenizer_tests;
mod tokenizer_with_feedback_tests;

use test::TestDescAndFn;

pub fn get_tests() -> Vec<TestDescAndFn> {
    let mut tests = Vec::new();

    macro_rules! add_tests {
        ($($m:ident),+) => {
            $(tests.extend($m::get_tests());)+
        };
    }

    add_tests!(
        tag_name_hash_tests,
        tokenizer_tests,
        tokenizer_with_feedback_tests
    );

    tests
}
