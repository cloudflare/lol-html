mod parsing_result;

use self::parsing_result::ParsingResult;
use super::TokenizerTestRunner;
use cool_thing::tokenizer::TextParsingModeSnapshot;
use harness::tokenizer_test::TokenizerTest;

/// Tests switching between state machines by parsing tags by both
/// of them in the context of the same tokenizer run.
pub struct StateMachineSwitchTestRunner;

impl TokenizerTestRunner for StateMachineSwitchTestRunner {
    fn get_test_description_suffix() -> &'static str {
        "State machine switch"
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

            assert_eql!(
                actual.tokens_from_preview,
                test.expected_tag_tokens,
                test.input,
                initial_mode_snapshot,
                "Tokens from preview mismatch"
            );
        }
    }
}
