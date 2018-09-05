use super::decoder::Decoder;
use super::token::TestToken;
use cool_thing::lex_unit::LexUnit;
use cool_thing::tokenizer::{TextParsingMode, TextParsingModeSnapshot};

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
}

impl ParsingResult {
    pub fn add_lex_unit(&mut self, lex_unit: &LexUnit, mode_snapshot: TextParsingModeSnapshot) {
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

    pub fn get_cumulative_raw_string(&self) -> String {
        self.raw_strings.iter().fold(String::new(), |c, s| c + s)
    }

    pub fn get_tokens(&self) -> &Vec<TestToken> {
        &self.tokens
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
