mod chunked_input;
mod decoder;
mod lex_unit_sink;
mod suits;
mod test_case;
mod test_outputs;
mod unescape;

use self::unescape::Unescape;
use cool_thing::tokenizer::{TagName, TextParsingMode, TextParsingModeSnapshot};
use std::fmt::Write;

pub use self::chunked_input::ChunkedInput;
pub use self::lex_unit_sink::LexUnitSink;
pub use self::suits::TEST_CASES;
pub use self::test_case::*;
pub use self::test_outputs::*;

pub const BUFFER_SIZE: usize = 2048;

pub fn get_tag_tokens(tokens: &[TestToken]) -> Vec<TestToken> {
    tokens
        .to_owned()
        .into_iter()
        .filter(|t| match t {
            TestToken::StartTag { .. } | TestToken::EndTag { .. } => true,
            _ => false,
        })
        .collect::<Vec<_>>()
}

pub trait TestFixture {
    fn get_test_description_suffix() -> &'static str;
    fn run_test_case(test: &TestCase, initial_mode_snapshot: TextParsingModeSnapshot);

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
            let initial_mode_snapshot = TextParsingModeSnapshot {
                mode: TextParsingMode::from(cs.as_str()),
                last_start_tag_name_hash: TagName::get_hash(&test.last_start_tag),
            };

            Self::run_test_case(test, initial_mode_snapshot);
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
