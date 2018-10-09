mod decoder;
mod parsing_result;
mod token;
mod unescape;

use self::parsing_result::ParsingResult;
pub use self::token::TestToken;
use self::unescape::Unescape;
use cool_thing::lex_unit::LexUnit;
use cool_thing::tag_name::TagName;
use cool_thing::tokenizer::{TextParsingMode, TextParsingModeSnapshot, Tokenizer};
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;

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

pub fn default_initial_states() -> Vec<String> {
    vec![String::from("Data state")]
}

#[derive(Deserialize, Default, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bailout {
    pub reason: String,
    pub parsed_chunk: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenizerTest {
    pub description: String,
    pub input: String,

    #[serde(rename = "output")]
    pub expected_tokens: Vec<TestToken>,

    #[serde(default = "default_initial_states")]
    pub initial_states: Vec<String>,

    #[serde(default)]
    pub double_escaped: bool,

    #[serde(default)]
    pub last_start_tag: String,

    #[serde(skip)]
    pub ignored: bool,

    #[serde(skip)]
    pub expected_bailout: Option<Bailout>,
}

impl Unescape for TokenizerTest {
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

fn parse(input: &[u8], initial_mode_snapshot: TextParsingModeSnapshot) -> ParsingResult {
    let mut result = ParsingResult::default();
    let mut bailout_reason = None;

    {
        let mode_snapshot = Rc::new(RefCell::new(TextParsingModeSnapshot {
            mode: TextParsingMode::Data,
            last_start_tag_name_hash: None,
        }));

        let mode_snapshot_rc = Rc::clone(&mode_snapshot);

        let text_parsing_mode_change_handler =
            Box::new(move |s| *mode_snapshot_rc.borrow_mut() = s);

        let mut tokenizer = Tokenizer::new(4095, |lex_unit: &LexUnit| {
            result.add_lex_unit(lex_unit, *mode_snapshot.borrow())
        });

        tokenizer.set_text_parsing_mode_change_handler(text_parsing_mode_change_handler);
        tokenizer.set_state(initial_mode_snapshot.mode.into());
        tokenizer.set_last_start_tag_name_hash(initial_mode_snapshot.last_start_tag_name_hash);

        tokenizer
            .write(input)
            .unwrap_or_else(|e| bailout_reason = Some(e));
    }

    if let Some(reason) = bailout_reason {
        result.add_bailout(reason);
    }

    result
}

impl TokenizerTest {
    pub fn init(&mut self) {
        self.ignored = self.unescape().is_err();

        // NOTE: tokenizer should always produce EOF token
        self.expected_tokens.push(TestToken::Eof);
    }

    fn assert_tokens_have_correct_raw_strings(&self, actual: ParsingResult) {
        if let Some(token_raw_pairs) = actual.into_token_raw_pairs() {
            for (token, raw, text_parsing_mode_snapshot) in token_raw_pairs {
                let mut actual = parse(raw.as_bytes(), text_parsing_mode_snapshot);

                assert_eql!(
                    *actual.get_tokens(),
                    vec![token.to_owned(), TestToken::Eof],
                    raw,
                    text_parsing_mode_snapshot,
                    "Token's raw string doesn't produce the same token"
                );
            }
        }
    }

    pub fn run(&self) {
        for cs in &self.initial_states {
            let cs = TextParsingMode::from(cs.as_str());

            let actual = parse(
                self.input.as_bytes(),
                TextParsingModeSnapshot {
                    mode: cs,
                    last_start_tag_name_hash: TagName::get_hash(&self.last_start_tag),
                },
            );

            assert_eql!(
                *actual.get_bailout(),
                self.expected_bailout,
                self.input,
                cs,
                "Tokenizer bailout error mismatch"
            );

            if actual.get_bailout().is_none() {
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

                self.assert_tokens_have_correct_raw_strings(actual);
            }
        }
    }
}