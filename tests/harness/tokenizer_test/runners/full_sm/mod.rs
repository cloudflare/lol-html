mod parsing_result;

use self::parsing_result::ParsingResult;
use super::TokenizerTestRunner;
use cool_thing::tokenizer::TextParsingModeSnapshot;
use harness::tokenizer_test::{TestToken, TokenizerTest};

/// Tests that full state machine produces correct lex units.
pub struct FullStateMachineTestRunner;

impl FullStateMachineTestRunner {
    fn assert_tokens_have_correct_raw_strings(actual: ParsingResult) {
        if let Some(token_raw_pairs) = actual.into_token_raw_pairs() {
            for (token, raw, text_parsing_mode_snapshot) in token_raw_pairs {
                let raw = raw.into();
                let mut actual = ParsingResult::new(&raw, text_parsing_mode_snapshot);

                assert_eql!(
                    actual.tokens,
                    vec![token.to_owned(), TestToken::Eof],
                    raw,
                    text_parsing_mode_snapshot,
                    "Token's raw string doesn't produce the same token"
                );
            }
        }
    }
}

impl TokenizerTestRunner for FullStateMachineTestRunner {
    fn get_test_description_suffix() -> &'static str {
        "Full state machine"
    }

    fn run_test_case(test: &TokenizerTest, initial_mode_snapshot: TextParsingModeSnapshot) {
        let actual = ParsingResult::new(&test.input, initial_mode_snapshot);

        assert_eql!(
            actual.bailout,
            test.expected_bailout,
            test.input,
            initial_mode_snapshot,
            "Tokenizer bailout error mismatch"
        );

        if actual.bailout.is_none() {
            assert_eql!(
                actual.tokens,
                test.expected_tokens,
                test.input,
                initial_mode_snapshot,
                "Token mismatch"
            );

            assert_eql!(
                actual.get_cumulative_raw_string(),
                test.input,
                test.input,
                initial_mode_snapshot,
                "Cumulative raw strings mismatch"
            );

            Self::assert_tokens_have_correct_raw_strings(actual);
        }
    }
}
