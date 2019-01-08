mod chunked_input;
mod decoder;
mod suits;
mod test_case;
mod test_token;
mod unescape;

use self::unescape::Unescape;
use cool_thing::tokenizer::{TagName, TextParsingMode};
use std::fmt::Write;

pub use self::chunked_input::ChunkedInput;
pub use self::suits::TEST_CASES;
pub use self::test_case::*;
pub use self::test_token::*;

// TODO functional test
pub trait TestFixture {
    fn get_test_description_suffix() -> &'static str;

    fn run_test_case(
        test: &TestCase,
        initial_mode: TextParsingMode,
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
                TextParsingMode::from(cs.as_str()),
                TagName::get_hash(&test.last_start_tag),
            );
        }
    }
}

macro_rules! assert_eql {
    ($actual:expr, $expected:expr, $cs:expr, $input:expr, $msg:expr) => {
        assert!(
            $actual == $expected,
            "{}\n\
             state: {:?}\n\
             input: {:?}\n\
             actual: {:#?}\n\
             expected: {:#?}",
            $msg,
            $input,
            $cs,
            $actual,
            $expected
        );
    };
}

macro_rules! tokenizer_test_fixture {
    ($fixture:ident) => {
        use crate::harness::tokenizer_test::TEST_CASES;
        use test::TestDescAndFn;

        pub fn get_tests() -> Vec<TestDescAndFn> {
            TEST_CASES
                .iter()
                .cloned()
                .map(|t| {
                    create_test!($fixture::get_test_description(&t), t.ignored, {
                        $fixture::run(&t);
                    })
                })
                .collect()
        }
    };
}
