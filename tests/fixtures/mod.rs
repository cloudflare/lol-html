mod eager_sm_tests;
mod full_sm_tests;
mod sm_switch_tests;
mod tag_name_hash_tests;
mod token_capturing_tests;

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
        eager_sm_tests,
        full_sm_tests,
        sm_switch_tests,
        token_capturing_tests
    );

    tests
}
