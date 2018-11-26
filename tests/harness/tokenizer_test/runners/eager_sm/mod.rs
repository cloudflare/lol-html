mod parsing_result;

use self::parsing_result::ParsingResult;
use super::TokenizerTestRunner;
use cool_thing::tokenizer::TextParsingModeSnapshot;
use harness::tokenizer_test::TokenizerTest;

/// Tests that eager state machine produces correct tag previews.
pub struct EagerStateMachineTestRunner;

impl TokenizerTestRunner for EagerStateMachineTestRunner {
    fn get_test_description_suffix() -> &'static str {
        "Eager state machine"
    }

    fn run_test_case(test: &TokenizerTest, initial_mode_snapshot: TextParsingModeSnapshot) {
        let actual = ParsingResult::new(&test.input, initial_mode_snapshot);

        if !actual.has_bailout {
            assert_eql!(
                actual.previews,
                test.expected_tag_tokens,
                test.input,
                initial_mode_snapshot,
                "Previews and tokens mismatch"
            );
        }
    }
}
