mod attributes;
mod tag_name_hash_tests;
mod token_capture;

use test::TestDescAndFn;

pub fn get_tests() -> Vec<TestDescAndFn> {
    let mut tests = Vec::new();

    macro_rules! add_tests {
        ($($m:ident),+) => {
            $(tests.extend($m::get_tests());)+
        };
    }

    add_tests!(tag_name_hash_tests, attributes, token_capture);

    tests
}
