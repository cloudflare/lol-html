mod parsing_result;

use self::parsing_result::ParsingResult;
use super::TokenizerTestRunner;
use cool_thing::tokenizer::TextParsingModeSnapshot;
use harness::tokenizer_test::TestToken;
use harness::tokenizer_test::TokenizerTest;

fn get_descendants_of_top_level_elements(tokens: &[TestToken]) -> Vec<Vec<TestToken>> {
    tokens
        .to_owned()
        .into_iter()
        .fold(
            (Vec::new(), Vec::new(), None, 0),
            |(
                mut captures,
                mut pending_token_set,
                captured_tag_name,
                mut open_captured_tag_count,
            ): (Vec<Vec<_>>, Vec<_>, Option<_>, usize),
             t| {
                macro_rules! add_pending_token_set {
                    () => {
                        if !pending_token_set.is_empty() {
                            captures.push(pending_token_set);
                            pending_token_set = Vec::new();
                        }
                    };
                }

                let captured_tag_name = match captured_tag_name {
                    Some(captured_tag_name) => match t {
                        TestToken::StartTag { ref name, .. } if *name == captured_tag_name => {
                            open_captured_tag_count += 1;
                            pending_token_set.push(t.to_owned());

                            Some(captured_tag_name)
                        }
                        TestToken::EndTag { ref name, .. } if *name == captured_tag_name => {
                            open_captured_tag_count -= 1;

                            if open_captured_tag_count == 0 {
                                add_pending_token_set!();
                                None
                            } else {
                                pending_token_set.push(t.to_owned());
                                Some(captured_tag_name)
                            }
                        }
                        TestToken::Eof => {
                            add_pending_token_set!();

                            None
                        }
                        _ => {
                            pending_token_set.push(t.to_owned());
                            Some(captured_tag_name)
                        }
                    },
                    None => match t {
                        TestToken::StartTag { name, .. } => {
                            open_captured_tag_count = 1;

                            Some(name.to_owned())
                        }
                        _ => None,
                    },
                };

                (
                    captures,
                    pending_token_set,
                    captured_tag_name,
                    open_captured_tag_count,
                )
            },
        ).0
}

/// Tests that tokenizer correctly captures lex units that
/// are descendants of the top level elements.
pub struct ContentCapturingTestRunner;

impl TokenizerTestRunner for ContentCapturingTestRunner {
    fn get_test_description_suffix() -> &'static str {
        "Content capturing"
    }

    fn run_test_case(test: &TokenizerTest, initial_mode_snapshot: TextParsingModeSnapshot) {
        let actual = ParsingResult::new(&test.input, initial_mode_snapshot);
        let expected_token_sets = get_descendants_of_top_level_elements(&test.expected_tokens);

        if !actual.has_bailout {
            assert_eql!(
                actual.token_sets,
                expected_token_sets,
                test.input,
                initial_mode_snapshot,
                "Token sets mismatch"
            );
        }
    }
}
