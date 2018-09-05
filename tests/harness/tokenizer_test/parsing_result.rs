use super::decoder::Decoder;
use super::token::TestToken;
use cool_thing::lex_unit::LexUnit;
use cool_thing::tokenizer::TextParsingMode;

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
    token_text_parsing_modes: Vec<TextParsingMode>,
    raw_strings: Vec<String>,
}

impl ParsingResult {
    pub fn add_lex_res(&mut self, lex_res: LexUnit, text_parsing_mode: TextParsingMode) {
        if let Some(token) = lex_res.as_token() {
            let token = (token, &lex_res).into();

            if let Some(TestToken::Character(ref mut prev_text)) = self.tokens.last_mut() {
                if let TestToken::Character(ref cur_text) = token {
                    *prev_text += cur_text;

                    if let Some(prev_raw) = self.raw_strings.last_mut() {
                        *prev_raw += cur_text;
                    }

                    return;
                } else {
                    *prev_text = decode_text(prev_text, text_parsing_mode);
                }
            }

            self.tokens.push(token);
            self.token_text_parsing_modes.push(text_parsing_mode);
        }

        if let Some(raw) = lex_res.raw {
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

    pub fn into_token_raw_pairs(mut self) -> Option<Vec<(TestToken, String, TextParsingMode)>> {
        // NOTE: remove EOF which doesn't have raw representation
        self.tokens.pop();

        // NOTE: we can build list of pairs only if each
        // token has a raw representation.
        if self.tokens.len() == self.raw_strings.len() {
            Some(
                izip!(
                    self.tokens.into_iter(),
                    self.raw_strings.into_iter(),
                    self.token_text_parsing_modes.into_iter()
                ).collect(),
            )
        } else {
            None
        }
    }
}
