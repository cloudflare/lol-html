use super::chunked_input::ChunkedInput;
use super::decoder::Decoder;
use super::token::TestToken;
use super::Bailout;
use cool_thing::tokenizer::{
    LexUnit, TextParsingMode, TextParsingModeSnapshot, TokenView, Tokenizer, TokenizerBailoutReason,
};
use std::cell::Cell;
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
    raw_slices: Vec<Vec<u8>>,
    bailout: Option<Bailout>,
    buffered_chars: Option<Vec<u8>>,
}

impl ParsingResult {
    pub fn new(input: &ChunkedInput, initial_mode_snapshot: TextParsingModeSnapshot) -> Self {
        let mut result = ParsingResult {
            tokens: Vec::new(),
            text_parsing_mode_snapshots: Vec::new(),
            raw_slices: Vec::new(),
            bailout: None,
            buffered_chars: None,
        };

        if let Err(e) = result.parse(input, initial_mode_snapshot) {
            result.add_bailout(e);
        }

        result
    }

    fn parse(
        &mut self,
        input: &ChunkedInput,
        initial_mode_snapshot: TextParsingModeSnapshot,
    ) -> Result<(), TokenizerBailoutReason> {
        let mode_snapshot = Rc::new(Cell::new(TextParsingModeSnapshot {
            mode: TextParsingMode::Data,
            last_start_tag_name_hash: None,
        }));

        let mode_snapshot_rc = Rc::clone(&mode_snapshot);
        let text_parsing_mode_change_handler = Box::new(move |s| mode_snapshot_rc.set(s));

        let mut tokenizer =
            Tokenizer::new(|lex_unit: &LexUnit| self.add_lex_unit(lex_unit, mode_snapshot.get()));

        tokenizer.set_text_parsing_mode_change_handler(text_parsing_mode_change_handler);
        tokenizer.set_state(initial_mode_snapshot.mode.into());
        tokenizer.set_last_start_tag_name_hash(initial_mode_snapshot.last_start_tag_name_hash);

        for chunk in input.get_chunks() {
            tokenizer.tokenize_chunk(&chunk.into())?;
        }

        tokenizer.finish();

        Ok(())
    }

    fn add_buffered_chars(&mut self, mode_snapshot: TextParsingModeSnapshot) {
        if let Some(buffered_chars) = self.buffered_chars.take() {
            let mut text = String::from_utf8(buffered_chars).unwrap();

            text = decode_text(&mut text, mode_snapshot.mode);
            self.tokens.push(TestToken::Character(text));
        }
    }

    fn buffer_chars(&mut self, chars: &[u8], mode_snapshot: TextParsingModeSnapshot) {
        if let Some(ref mut buffered_chars) = self.buffered_chars {
            buffered_chars.extend_from_slice(chars);

            if let Some(last_raw) = self.raw_slices.last_mut() {
                last_raw.extend_from_slice(chars);
            }
        } else {
            self.buffered_chars = Some(chars.to_vec());
            self.raw_slices.push(chars.to_vec());
            self.text_parsing_mode_snapshots.push(mode_snapshot);
        }
    }

    fn add_lex_unit(&mut self, lex_unit: &LexUnit, mode_snapshot: TextParsingModeSnapshot) {
        if let (Some(TokenView::Character), Some(raw)) =
            (lex_unit.get_token_view(), lex_unit.get_raw())
        {
            self.buffer_chars(&raw, mode_snapshot);
        } else {
            if let Some(token) = lex_unit.get_token() {
                self.add_buffered_chars(mode_snapshot);
                self.tokens.push((token, lex_unit).into());
                self.text_parsing_mode_snapshots.push(mode_snapshot);
            }

            if let Some(raw) = lex_unit.get_raw() {
                self.raw_slices.push(raw.to_vec());
            }
        }
    }

    fn add_bailout(&mut self, reason: TokenizerBailoutReason) {
        self.bailout = Some(Bailout {
            reason: format!("{:?}", reason),
            parsed_chunk: self.get_cumulative_raw_string(),
        });
    }

    pub fn get_cumulative_raw_string(&self) -> String {
        String::from_utf8(self.raw_slices.iter().fold(Vec::new(), |mut c, s| {
            c.extend_from_slice(s);
            c
        })).unwrap()
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

        // NOTE: there are cases there character token can contain
        // part that is ignored during parsing, but still has raw
        // representation. E.g. `a</>b` will produce `ab` character
        // token, however we'll have `a` and `</>b` raw strings and,
        // thus, we can't produce one-on-one mapping.
        if self.tokens.len() == self.raw_slices.len() {
            Some(
                izip!(
                    self.tokens.into_iter(),
                    self.raw_slices
                        .into_iter()
                        .map(|s| String::from_utf8(s).unwrap()),
                    self.text_parsing_mode_snapshots.into_iter()
                ).collect(),
            )
        } else {
            None
        }
    }
}
