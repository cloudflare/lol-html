use cool_thing::tokenizer::{TagName, TextParsingMode, TextParsingModeSnapshot};
use harness::tokenizer_test::TokenizerTest;
use std::fmt::Write;

const BUFFER_SIZE: usize = 2048;

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

mod eager_sm;
mod full_sm;
mod sm_switch;

pub use self::eager_sm::EagerStateMachineTestRunner;
pub use self::full_sm::FullStateMachineTestRunner;
pub use self::sm_switch::StateMachineSwitchTestRunner;

pub trait TokenizerTestRunner {
    fn get_test_description_suffix() -> &'static str;
    fn run_test_case(test: &TokenizerTest, initial_mode_snapshot: TextParsingModeSnapshot);

    fn get_test_description(test: &TokenizerTest) -> String {
        let mut descr = String::new();

        write!(
            &mut descr,
            "{} - {}",
            test.description,
            Self::get_test_description_suffix()
        ).unwrap();

        descr
    }

    fn run(test: &TokenizerTest) {
        for cs in &test.initial_states {
            let initial_mode_snapshot = TextParsingModeSnapshot {
                mode: TextParsingMode::from(cs.as_str()),
                last_start_tag_name_hash: TagName::get_hash(&test.last_start_tag),
            };

            Self::run_test_case(test, initial_mode_snapshot);
        }
    }
}
