mod decoder;
mod test_cases;
mod test_token;

use crate::harness::parsing::ChunkedInput;
use cool_thing::parser::{TagName, TextType};
use std::fmt::Write;

pub use self::test_cases::*;
pub use self::test_token::*;

pub trait FunctionalTestFixture {
    fn get_test_description_suffix() -> &'static str;

    fn run_test_case(
        test: &TestCase,
        initial_text_type: TextType,
        last_start_tag_name_hash: Option<u64>,
    );

    fn get_test_description(test: &TestCase) -> String {
        let mut descr = String::new();

        write!(
            &mut descr,
            "{} - {}",
            test.description,
            Self::get_test_description_suffix()
        )
        .unwrap();

        descr
    }

    fn run(test: &TestCase) {
        for cs in &test.initial_states {
            Self::run_test_case(
                test,
                TextType::from(cs.as_str()),
                TagName::get_hash(&test.last_start_tag),
            );
        }
    }
}

macro_rules! expect_eql {
    ($actual:expr, $expected:expr, $state:expr, $input:expr, $msg:expr) => {
        assert!(
            $actual == $expected,
            "{}\n\
             actual: {:#?}\n\
             expected: {:#?}\n\
             state: {:?}\n\
             input: {:?}\n\
             ",
            $msg,
            $actual,
            $expected,
            $state,
            $input,
        );
    };
}

macro_rules! expect {
    ($actual:expr, $state:expr, $input:expr, $msg:expr) => {
        assert!(
            $actual,
            "{}\n\
             state: {:?}\n\
             input: {:?}\n\
             ",
            $msg, $state, $input,
        );
    };
}

macro_rules! functional_test_fixture {
    ($fixture:ident) => {
        use crate::harness::functional_testing::TEST_CASES;
        use test::TestDescAndFn;

        pub fn get_tests() -> Vec<TestDescAndFn> {
            TEST_CASES
                .iter()
                .cloned()
                .map(|t| {
                    create_test!($fixture::get_test_description(&t), {
                        $fixture::run(&t);
                    })
                })
                .collect()
        }
    };
}
