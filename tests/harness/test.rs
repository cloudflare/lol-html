use super::decoder::Decoder;
use super::initial_state::InitialState;
use super::raw_string_vec::RawStringVec;
use super::unescape::Unescape;
use cool_thing::{get_tag_name_hash, LexResult, Tokenizer};
use serde_json;
use super::token::TestToken;

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

fn decode_text(text: &mut str, initial_state: InitialState) -> String {
    let mut decoder = Decoder::new(text);

    if initial_state.should_replace_unsafe_null_in_text() {
        decoder = decoder.unsafe_null();
    }

    if initial_state.allows_text_entitites() {
        decoder = decoder.text_entities();
    }

    decoder.run()
}

fn handle_lex_result(
    tokens: &mut Vec<TestToken>,
    raw_strings: &mut RawStringVec,
    initial_state: InitialState,
    lex_res: LexResult,
) {
    if let Some(token) = lex_res.as_token() {
        let token = TestToken::from(&token);

        if let Some(&mut TestToken::Character(ref mut prev)) = tokens.last_mut() {
            if let TestToken::Character(ref curr) = token {
                *prev += curr;

                if let Some(ref mut prev_raw) = raw_strings
                    .last_mut()
                    .expect("Raw string for previous token should exist")
                {
                    *prev_raw += curr;
                }

                return;
            } else {
                *prev = decode_text(prev, initial_state);
            }
        }

        tokens.push(token);
    }

    raw_strings.push_raw(lex_res.raw);
}

impl Test {
    pub fn init(&mut self) {
        self.ignored = self.unescape().is_err();

        // NOTE: tokenizer should always produce EOF token
        self.expected_tokens.push(TestToken::Eof);
    }

    fn parse(&self, input: Vec<u8>, initial_state: InitialState) -> (Vec<TestToken>, RawStringVec) {
        let mut tokens = Vec::new();
        let mut raw_strings = RawStringVec::default();

        {
            let mut tokenizer = Tokenizer::new(2048, |lex_res: LexResult| {
                handle_lex_result(&mut tokens, &mut raw_strings, initial_state, lex_res);
            });

            tokenizer.set_state(initial_state.to_tokenizer_state());
            tokenizer.set_last_start_tag_name_hash(get_tag_name_hash(&self.last_start_tag));

            tokenizer
                .write(input)
                .expect("Tokenizer buffer capacity exceeded");
        }

        (tokens, raw_strings)
    }

    pub fn run(&self) {
        for &cs in &self.initial_states {
            macro_rules! assert_eql {
                ($actual:expr, $expected:expr, $msg:expr) => {
                    assert!(
                        $actual == $expected,
                        "{}\n\
                         state: {:?}\n\
                         input: {:?}\n\
                         actual: {:#?}\n\
                         expected: {:#?}",
                        $msg,
                        cs,
                        self.input,
                        $actual,
                        $expected
                    );
                };
            };

            let (actual_tokens, raw_strings) = self.parse(self.input.bytes().collect(), cs);

            assert_eql!(actual_tokens, self.expected_tokens, "Token mismatch");

            assert_eql!(
                raw_strings.get_cumulative_raw_string(),
                self.input,
                "Cumulative raw strings mismatch"
            );
        }
    }
}
