use crate::harness::functional_testing::{
    FunctionalTestFixture, TestCase, TestToken, TestTokenList,
};
use crate::harness::parsing::{parse, ContentSettings};
use cool_thing::parser::TextType;
use cool_thing::token::TokenCaptureFlags;

fn filter_tokens(tokens: &[TestToken], capture_flags: TokenCaptureFlags) -> Vec<TestToken> {
    tokens
        .iter()
        .cloned()
        .filter(|t| match t {
            TestToken::Doctype { .. } if capture_flags.contains(TokenCaptureFlags::DOCTYPES) => {
                true
            }
            TestToken::StartTag { .. } if capture_flags.contains(TokenCaptureFlags::START_TAGS) => {
                true
            }
            TestToken::EndTag { .. } if capture_flags.contains(TokenCaptureFlags::END_TAGS) => true,
            TestToken::Comment(_) if capture_flags.contains(TokenCaptureFlags::COMMENTS) => true,
            TestToken::Text(_) if capture_flags.contains(TokenCaptureFlags::TEXT) => true,
            _ => false,
        })
        .collect()
}

fn fold_text_tokens(tokens: Vec<TestToken>) -> Vec<TestToken> {
    tokens.into_iter().fold(Vec::new(), |mut res, t| {
        if let TestToken::Text(ref text) = t {
            if let Some(TestToken::Text(last)) = res.last_mut() {
                *last += text;

                return res;
            }
        }

        res.push(t);

        res
    })
}

pub struct TokenCapturerTests;

impl FunctionalTestFixture for TokenCapturerTests {
    fn get_test_description_suffix() -> &'static str {
        "Token capture"
    }

    fn run_test_case(
        test: &TestCase,
        initial_text_type: TextType,
        last_start_tag_name_hash: Option<u64>,
    ) {
        [
            (ContentSettings::all(), TokenCaptureFlags::all()),
            (ContentSettings::start_tags(), TokenCaptureFlags::START_TAGS),
            (ContentSettings::end_tags(), TokenCaptureFlags::END_TAGS),
            (ContentSettings::text(), TokenCaptureFlags::TEXT),
            (ContentSettings::comments(), TokenCaptureFlags::COMMENTS),
            (ContentSettings::doctypes(), TokenCaptureFlags::DOCTYPES),
        ]
        .iter()
        .cloned()
        .for_each(|(content_settings, expected_token_flags)| {
            let mut expected_tokens = filter_tokens(&test.expected_tokens, expected_token_flags);
            let mut token_list = TestTokenList::default();

            let parsing_result = parse(
                &test.input,
                content_settings,
                initial_text_type,
                last_start_tag_name_hash,
                Box::new(|t| token_list.push(t)),
            );

            let mut actual_tokens = token_list.into();

            // NOTE: text is a special case: it's impossible to achieve the same
            // text chunks layout as in the test data without surrounding tokens
            // (in test data all character tokens that are not separated by other
            // tokens get concatenated, ignoring any non-token lexems like `<![CDATA[`
            // in-between). On the contrary we break character token chain on non-token
            // lexems and, therefore, if non-token lexems are present we won't get the
            // same character token layout as in test data if we just concatenate all
            // tokens in the chain. So, for text tokens we fold both expected and actual
            // results to the single strings. It's not an ideal solution, but it's better
            // than nothing.
            if expected_token_flags == TokenCaptureFlags::TEXT {
                actual_tokens = fold_text_tokens(actual_tokens);
                expected_tokens = fold_text_tokens(expected_tokens);
            }

            match parsing_result {
                Ok(output) => {
                    expect_eql!(
                        actual_tokens,
                        expected_tokens,
                        initial_text_type,
                        test.input,
                        format!("Token mismatch (capture: {:#?})", expected_token_flags)
                    );

                    expect_eql!(
                        output,
                        test.input.as_str(),
                        initial_text_type,
                        test.input,
                        format!(
                            "Serialized output doesn't match original input (capture: {:#?})",
                            expected_token_flags
                        )
                    );
                }
                Err(_) => {
                    expect!(
                        test.expected_bailout.is_some(),
                        initial_text_type,
                        test.input,
                        format!("Unexpected bailout (capture: {:#?})", expected_token_flags)
                    );
                }
            }
        });
    }
}

functional_test_fixture!(TokenCapturerTests);
