use super::initial_state::InitialState;
use super::parsing_result::ParsingResult;
use super::token::TestToken;
use super::unescape::Unescape;
use cool_thing::{get_tag_name_hash, LexResult, Tokenizer};
use serde_json;

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

fn default_initial_states() -> Vec<InitialState> {
    vec![InitialState::Data]
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Test {
    pub description: String,
    pub input: String,

    #[serde(rename = "output")]
    pub expected_tokens: Vec<TestToken>,

    #[serde(skip)]
    pub ignored: bool,

    #[serde(default = "default_initial_states")]
    pub initial_states: Vec<InitialState>,

    #[serde(default)]
    pub double_escaped: bool,

    #[serde(default)]
    pub last_start_tag: String,
}

impl Unescape for Test {
    fn unescape(&mut self) -> Result<(), serde_json::error::Error> {
        if self.double_escaped {
            self.double_escaped = false;
            self.input.unescape()?;

            for token in &mut self.expected_tokens {
                token.unescape()?;
            }
        }

        Ok(())
    }
}

impl Test {
    pub fn init(&mut self) {
        self.ignored = self.unescape().is_err();

        // NOTE: tokenizer should always produce EOF token
        self.expected_tokens.push(TestToken::Eof);
    }

    fn parse(&self, input: Vec<u8>, initial_state: InitialState) -> ParsingResult {
        let mut result = ParsingResult::new(initial_state);

        {
            let mut tokenizer = Tokenizer::new(2048, |lex_res: LexResult| {
                result.add_lex_res(lex_res);
            });

            tokenizer.set_state(initial_state.to_tokenizer_state());
            tokenizer.set_last_start_tag_name_hash(get_tag_name_hash(&self.last_start_tag));

            tokenizer
                .write(input)
                .expect("Tokenizer buffer capacity exceeded");
        }

        result
    }

    pub fn run(&self) {
        for &cs in &self.initial_states {
            let actual = self.parse(self.input.bytes().collect(), cs);

            assert_eql!(
                *actual.get_tokens(),
                self.expected_tokens,
                self.input,
                cs,
                "Token mismatch"
            );

            assert_eql!(
                actual.get_cumulative_raw_string(),
                self.input,
                self.input,
                cs,
                "Cumulative raw strings mismatch"
            );

            // TODO: write raw strings one by one and check that the last token in result is equal
            // to current. We need streaming support for that.
            //self.assert_tokens_have_correct_raw_strings(actual, cs);
        }
    }
}
