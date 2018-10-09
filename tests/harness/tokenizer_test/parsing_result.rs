use super::decoder::Decoder;
use super::token::TestToken;
use super::Bailout;
use cool_thing::lex_unit::LexUnit;
use cool_thing::tokenizer::{
    TextParsingMode, TextParsingModeSnapshot, Tokenizer, TokenizerBailoutReason,
};
use std::cell::RefCell;
use std::rc::Rc;

fn decode_text(text: &mut str, initial_state: TextParsingMode) -> String {
    let mut decoder = Decoder::new(text);

    if initial_state.should_replace_unsafe_null_in_text() {
        decoder = decoder.unsafe_null();
    }

    if initial_state.allows_text_entitites() {
        decoder = decoder.text_entities();
    }

    decoder.run()
}

#[derive(Default)]
pub struct ParsingResult {
    tokens: Vec<TestToken>,
    text_parsing_mode_snapshots: Vec<TextParsingModeSnapshot>,
    raw_strings: Vec<String>,
    bailout: Option<Bailout>,
}

impl ParsingResult {
    pub fn new(input: &[u8], initial_mode_snapshot: TextParsingModeSnapshot) -> Self {
        let mut result = ParsingResult {
            tokens: Vec::new(),
            text_parsing_mode_snapshots: Vec::new(),
            raw_strings: Vec::new(),
            bailout: None,
        };

        result.parse(input, initial_mode_snapshot);

        result
    }

    fn parse(&mut self, input: &[u8], initial_mode_snapshot: TextParsingModeSnapshot) {
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
                self.add_lex_unit(lex_unit, *mode_snapshot.borrow())
            });

            tokenizer.set_text_parsing_mode_change_handler(text_parsing_mode_change_handler);
            tokenizer.set_state(initial_mode_snapshot.mode.into());
            tokenizer.set_last_start_tag_name_hash(initial_mode_snapshot.last_start_tag_name_hash);

            tokenizer
                .write(input)
                .unwrap_or_else(|e| bailout_reason = Some(e));
        }

        if let Some(reason) = bailout_reason {
            self.add_bailout(reason);
        }
    }

    fn add_lex_unit(&mut self, lex_unit: &LexUnit, mode_snapshot: TextParsingModeSnapshot) {
        if let Some(token) = lex_unit.as_token() {
            let token = (token, lex_unit).into();

            if let Some(TestToken::Character(ref mut prev_text)) = self.tokens.last_mut() {
                if let TestToken::Character(ref cur_text) = token {
                    *prev_text += cur_text;

                    if let Some(prev_raw) = self.raw_strings.last_mut() {
                        *prev_raw += cur_text;
                    }

                    return;
                } else {
                    *prev_text = decode_text(prev_text, mode_snapshot.mode);
                }
            }

            self.tokens.push(token);

            self.text_parsing_mode_snapshots.push(mode_snapshot);
        }

        if let Some(raw) = lex_unit.raw {
            self.raw_strings
                .push(unsafe { String::from_utf8_unchecked(raw.to_vec()) });
        }
    }

    fn add_bailout(&mut self, reason: TokenizerBailoutReason) {
        self.bailout = Some(Bailout {
            reason: format!("{:?}", reason),
            parsed_chunk: self.get_cumulative_raw_string(),
        });
    }

    pub fn get_cumulative_raw_string(&self) -> String {
        self.raw_strings.iter().fold(String::new(), |c, s| c + s)
    }

    pub fn get_tokens(&self) -> &Vec<TestToken> {
        &self.tokens
    }

    pub fn get_bailout(&self) -> &Option<Bailout> {
        &self.bailout
    }

    pub fn into_token_raw_pairs(
        mut self,
    ) -> Option<Vec<(TestToken, String, TextParsingModeSnapshot)>> {
        // NOTE: remove EOF which doesn't have raw representation
        self.tokens.pop();

        // NOTE: we can build list of pairs only if each
        // token has a raw representation.
        if self.tokens.len() == self.raw_strings.len() {
            Some(
                izip!(
                    self.tokens.into_iter(),
                    self.raw_strings.into_iter(),
                    self.text_parsing_mode_snapshots.into_iter()
                ).collect(),
            )
        } else {
            None
        }
    }
}
