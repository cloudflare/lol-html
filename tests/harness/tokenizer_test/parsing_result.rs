use super::chunked_input::ChunkedInput;
use super::decoder::Decoder;
use super::tag_preview::TestTagPreview;
use super::token::TestToken;
use super::Bailout;
use cool_thing::tokenizer::{
    LexUnit, NextOutputType, TagPreview, TextParsingMode, TextParsingModeSnapshot, TokenView,
};
use cool_thing::transform_stream::TransformStream;
use cool_thing::Error;
use itertools::izip;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn decode_text(text: &mut str, text_parsing_mode: TextParsingMode) -> String {
    let mut decoder = Decoder::new(text);

    if text_parsing_mode.should_replace_unsafe_null_in_text() {
        decoder = decoder.unsafe_null();
    }

    if text_parsing_mode.allows_text_entitites() {
        decoder = decoder.text_entities();
    }

    decoder.run()
}

#[derive(Default)]
pub struct ParsingResult {
    tokens: Vec<TestToken>,
    tag_previews: Vec<TestTagPreview>,
    text_parsing_mode_snapshots: Vec<TextParsingModeSnapshot>,
    raw_slices: Vec<Vec<u8>>,
    bailout: Option<Bailout>,
    buffered_chars: Option<Vec<u8>>,
}

impl ParsingResult {
    pub fn new(input: &ChunkedInput, initial_mode_snapshot: TextParsingModeSnapshot) -> Self {
        let mut result = ParsingResult {
            tokens: Vec::new(),
            tag_previews: Vec::new(),
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
    ) -> Result<(), Error> {
        let result_rc1 = Rc::new(RefCell::new(self));
        let result_rc2 = Rc::clone(&result_rc1);
        let result_rc3 = Rc::clone(&result_rc1);

        let mode_snapshot_rc1 = Rc::new(Cell::new(TextParsingModeSnapshot::default()));
        let mode_snapshot_rc2 = Rc::clone(&mode_snapshot_rc1);
        let mode_snapshot_rc3 = Rc::clone(&mode_snapshot_rc1);

        let text_parsing_mode_change_handler = Box::new(move |s| mode_snapshot_rc1.set(s));

        let mut transform_stream = TransformStream::new(
            2048,
            move |lex_unit: &LexUnit| {
                result_rc1
                    .borrow_mut()
                    .add_lex_unit(lex_unit, mode_snapshot_rc2.get());
            },
            move |lex_unit: &LexUnit| {
                result_rc2
                    .borrow_mut()
                    .add_lex_unit(lex_unit, mode_snapshot_rc3.get());

                NextOutputType::LexUnit
            },
            move |tag_preview: &TagPreview| {
                result_rc3
                    .borrow_mut()
                    .tag_previews
                    .push(TestTagPreview::from_tag_preview(tag_preview));

                NextOutputType::TagPreview
            },
        );

        {
            let tokenizer = transform_stream.get_tokenizer();

            tokenizer.set_next_output_type(NextOutputType::LexUnit);
            tokenizer.set_text_parsing_mode_change_handler(text_parsing_mode_change_handler);
            tokenizer.set_text_parsing_mode(initial_mode_snapshot.mode);
            tokenizer.set_last_start_tag_name_hash(initial_mode_snapshot.last_start_tag_name_hash);
        }

        for chunk in input.get_chunks() {
            transform_stream.write(chunk)?;
        }

        transform_stream.end()?;

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
                self.tokens.push(TestToken::new(token, lex_unit));
                self.text_parsing_mode_snapshots.push(mode_snapshot);
            }

            if let Some(raw) = lex_unit.get_raw() {
                self.raw_slices.push(raw.to_vec());
            }
        }
    }

    fn add_bailout(&mut self, reason: Error) {
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

    pub fn get_tag_previews(&self) -> &Vec<TestTagPreview> {
        &self.tag_previews
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
