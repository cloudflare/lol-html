use crate::harness::functional_testing::html5lib_tests::{
    get_test_cases, TestCase, TestToken, TestTokenList,
};
use crate::harness::functional_testing::FunctionalTestFixture;
use crate::harness::parse;
use cool_thing::{TagName, TextType, TokenCaptureFlags};

macro_rules! expect_eql {
    ($actual:expr, $expected:expr, $state:expr, $input:expr, $msg:expr) => {
        assert!(
            $actual == $expected,
            "{}\n\
             actual: {:#?}\n\
             expected: {:#?}\n\
             state: {:?}\n\
             input: {:?}\n\
             ",
            $msg,
            $actual,
            $expected,
            $state,
            $input,
        );
    };
}

macro_rules! expect {
    ($actual:expr, $state:expr, $input:expr, $msg:expr) => {
        assert!(
            $actual,
            "{}\n\
             state: {:?}\n\
             input: {:?}\n\
             ",
            $msg, $state, $input,
        );
    };
}

fn filter_tokens(tokens: &[TestToken], capture_flags: TokenCaptureFlags) -> Vec<TestToken> {
    tokens
        .iter()
        .cloned()
        .filter(|t| match t {
            TestToken::Doctype { .. } if capture_flags.contains(TokenCaptureFlags::DOCTYPES) => {
                true
            }
            TestToken::StartTag { .. }
                if capture_flags.contains(TokenCaptureFlags::NEXT_START_TAG) =>
            {
                true
            }
            TestToken::EndTag { .. } if capture_flags.contains(TokenCaptureFlags::NEXT_END_TAG) => {
                true
            }
            TestToken::Comment(_) if capture_flags.contains(TokenCaptureFlags::COMMENTS) => true,
            TestToken::Text(_) if capture_flags.contains(TokenCaptureFlags::TEXT) => true,
            _ => false,
        })
        .collect()
}

fn fold_text_tokens(tokens: Vec<TestToken>) -> Vec<TestToken> {
    tokens.into_iter().fold(Vec::default(), |mut res, t| {
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

pub struct TokenCapturingTests;

impl TokenCapturingTests {
    fn run_test_case(
        test: &TestCase,
        initial_text_type: TextType,
        last_start_tag_name_hash: Option<u64>,
    ) {
        [
            TokenCaptureFlags::all(),
            TokenCaptureFlags::NEXT_START_TAG,
            TokenCaptureFlags::NEXT_END_TAG,
            TokenCaptureFlags::TEXT,
            TokenCaptureFlags::COMMENTS,
            TokenCaptureFlags::DOCTYPES,
            TokenCaptureFlags::empty(),
        ]
        .iter()
        .cloned()
        .for_each(|capture_flags| {
            let mut expected_tokens = filter_tokens(&test.expected_tokens, capture_flags);
            let mut token_list = TestTokenList::default();

            let parsing_result = parse(
                &test.input,
                capture_flags,
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
            if capture_flags == TokenCaptureFlags::TEXT {
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
                        format!("Token mismatch (capture: {:#?})", capture_flags)
                    );

                    expect_eql!(
                        output,
                        test.input.as_str(),
                        initial_text_type,
                        test.input,
                        format!(
                            "Serialized output doesn't match original input (capture: {:#?})",
                            capture_flags
                        )
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

impl FunctionalTestFixture<TestCase> for TokenCapturingTests {
    fn test_cases() -> Vec<TestCase> {
        get_test_cases()
    }

    fn run(test: &TestCase) {
        for cs in &test.initial_states {
            Self::run_test_case(
                test,
                TextType::from(cs.as_str()),
                TagName::get_hash(&test.last_start_tag),
            );
        }
    }
}

functional_test_fixture!(TokenCapturingTests);
