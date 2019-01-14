use crate::harness::functional_testing::{
    FunctionalTestFixture, TestCase, TestToken, TestTokenList,
};
use cool_thing::token::{Token, TokenCaptureFlags};
use cool_thing::tokenizer::{LexUnit, NextOutputType, TagPreview, TextType};
use cool_thing::transform_stream::TransformController;
use std::cell::RefCell;
use std::rc::Rc;

struct TestTransformController {
    token_list: Rc<RefCell<TestTokenList>>,
    capture_flags: TokenCaptureFlags,
}

impl TestTransformController {
    pub fn new(token_list: Rc<RefCell<TestTokenList>>, capture_flags: TokenCaptureFlags) -> Self {
        TestTransformController {
            token_list,
            capture_flags,
        }
    }
}

impl TransformController for TestTransformController {
    fn get_initial_token_capture_flags(&self) -> TokenCaptureFlags {
        self.capture_flags
    }

    fn get_token_capture_flags_for_tag(&mut self, _: &LexUnit) -> NextOutputType {
        NextOutputType::LexUnit
    }

    fn get_token_capture_flags_for_tag_preview(&mut self, _: &TagPreview) -> NextOutputType {
        NextOutputType::LexUnit
    }

    fn handle_token(&mut self, token: Token) {
        self.token_list.borrow_mut().push(token);
    }
}

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
            TestToken::Eof if capture_flags.contains(TokenCaptureFlags::EOF) => true,
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

pub struct TokenCaptureTests;

impl FunctionalTestFixture for TokenCaptureTests {
    fn get_test_description_suffix() -> &'static str {
        "Token capture"
    }

    fn run_test_case(
        test: &TestCase,
        initial_text_type: TextType,
        last_start_tag_name_hash: Option<u64>,
    ) {
        [
            TokenCaptureFlags::all(),
            TokenCaptureFlags::START_TAGS,
            TokenCaptureFlags::END_TAGS,
            TokenCaptureFlags::TEXT,
            TokenCaptureFlags::COMMENTS,
            TokenCaptureFlags::DOCTYPES,
            TokenCaptureFlags::empty(),
        ]
        .iter()
        .cloned()
        .for_each(|capture_flags| {
            let mut expected_tokens = filter_tokens(&test.expected_tokens, capture_flags);
            let token_list = Rc::new(RefCell::new(TestTokenList::default()));

            let transform_controller =
                TestTransformController::new(Rc::clone(&token_list), capture_flags);

            let parsing_result = test.input.parse(
                transform_controller,
                initial_text_type,
                last_start_tag_name_hash,
            );

            let mut actual_tokens = Rc::try_unwrap(token_list).unwrap().into_inner().into();

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
            if capture_flags == TokenCaptureFlags::TEXT {
                actual_tokens = fold_text_tokens(actual_tokens);
                expected_tokens = fold_text_tokens(expected_tokens);
            }

            match parsing_result {
                Ok(_) => {
                    expect_eql!(
                        actual_tokens,
                        expected_tokens,
                        initial_text_type,
                        test.input,
                        format!("Token mismatch (capture: {:#?})", capture_flags)
                    );
                }
                Err(_) => {
                    expect!(
                        test.expected_bailout.is_some(),
                        initial_text_type,
                        test.input,
                        format!("Unexpected bailout (capture: {:#?})", capture_flags)
                    );
                }
            }
        });
    }
}

functional_test_fixture!(TokenCaptureTests);
