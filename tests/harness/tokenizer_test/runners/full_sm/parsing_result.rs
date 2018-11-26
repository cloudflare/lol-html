use cool_thing::tokenizer::{
    LexUnit, NextOutputType, TagPreview, TextParsingMode, TextParsingModeSnapshot, TokenView,
};
use cool_thing::transform_stream::TransformStream;
use cool_thing::Error;
use harness::tokenizer_test::chunked_input::ChunkedInput;
use harness::tokenizer_test::decoder::Decoder;
use harness::tokenizer_test::runners::BUFFER_SIZE;
use harness::tokenizer_test::test_outputs::TestToken;
use harness::tokenizer_test::Bailout;
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
    pub tokens: Vec<TestToken>,
    pub bailout: Option<Bailout>,
    text_parsing_mode_snapshots: Vec<TextParsingModeSnapshot>,
    raw_slices: Vec<Vec<u8>>,
    buffered_chars: Option<Vec<u8>>,
}

impl ParsingResult {
    pub fn new(input: &ChunkedInput, initial_mode_snapshot: TextParsingModeSnapshot) -> Self {
        let mut result = ParsingResult {
            tokens: Vec::new(),
            bailout: None,
            text_parsing_mode_snapshots: Vec::new(),
            raw_slices: Vec::new(),
            buffered_chars: None,
        };

        // TODO use bailout handler later with substitution and test eager state machine as well.
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
        let result = Rc::new(RefCell::new(self));
        let mode_snapshot = Rc::new(Cell::new(TextParsingModeSnapshot::default()));

        let lex_unit_handler = {
            let result = Rc::clone(&result);
            let mode_snapshot = Rc::clone(&mode_snapshot);

            move |lex_unit: &LexUnit| {
                result
                    .borrow_mut()
                    .add_lex_unit(lex_unit, mode_snapshot.get());
            }
        };

        let tag_lex_unit_handler = {
            let result = Rc::clone(&result);
            let mode_snapshot = Rc::clone(&mode_snapshot);

            move |lex_unit: &LexUnit| {
                result
                    .borrow_mut()
                    .add_lex_unit(lex_unit, mode_snapshot.get());

                NextOutputType::LexUnit
            }
        };

        let tag_preview_handler = |_: &TagPreview| NextOutputType::TagPreview;

        let mut transform_stream = TransformStream::new(
            BUFFER_SIZE,
            lex_unit_handler,
            tag_lex_unit_handler,
            tag_preview_handler,
        );

        transform_stream
            .get_tokenizer()
            .get_full_sm()
            .set_text_parsing_mode_change_handler({
                let mode_snapshot = Rc::clone(&mode_snapshot);

                Box::new(move |s| mode_snapshot.set(s))
            });

        input.parse(
            transform_stream,
            initial_mode_snapshot,
            NextOutputType::LexUnit,
        )
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

    fn add_bailout(&mut self, reason: Error) {
        self.bailout = Some(Bailout {
            reason: format!("{:?}", reason),
            parsed_chunk: self.get_cumulative_raw_string(),
        });
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

    pub fn get_cumulative_raw_string(&self) -> String {
        String::from_utf8(self.raw_slices.iter().fold(Vec::new(), |mut c, s| {
            c.extend_from_slice(s);
            c
        })).unwrap()
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
