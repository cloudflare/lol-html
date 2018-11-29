mod parsing_result;

use self::parsing_result::ParsingResult;
use super::{get_tag_tokens, TokenizerTestRunner};
use cool_thing::tokenizer::TextParsingModeSnapshot;
use harness::tokenizer_test::TokenizerTest;

/// Tests that eager state machine produces correct tag previews.
pub struct EagerStateMachineTestRunner;

// TODO
// 1. combine runners and parsing result
// 2. move what's in fixtures to test cases
// 3. move runners to fixtures, rename runner to tokenizer test fixture
impl TokenizerTestRunner for EagerStateMachineTestRunner {
    fn get_test_description_suffix() -> &'static str {
        "Eager state machine"
    }

    fn run_test_case(test: &TokenizerTest, initial_mode_snapshot: TextParsingModeSnapshot) {
        let actual = ParsingResult::new(&test.input, initial_mode_snapshot);
        let expected_tokens = get_tag_tokens(&test.expected_tokens);

        if !actual.has_bailout {
            assert_eql!(
                actual.previews,
                expected_tokens,
                test.input,
                initial_mode_snapshot,
                "Previews and tokens mismatch"
            );
        }
    }
}
