use cool_thing::base::Bytes;
use cool_thing::tokenizer::{LexUnit, NextOutputType, TagPreview, TextParsingModeSnapshot, Token};
use cool_thing::transform_stream::TransformStream;
use cool_thing::Error;
use harness::tokenizer_test::chunked_input::ChunkedInput;
use harness::tokenizer_test::lex_unit_sink::LexUnitSink;
use harness::tokenizer_test::runners::BUFFER_SIZE;
use harness::tokenizer_test::test_outputs::TestToken;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn to_lower_string(bytes: &Bytes<'_>) -> String {
    let mut string = bytes.as_string();

    string.make_ascii_lowercase();

    string
}

pub struct ParsingResult {
    pub token_sets: Vec<Vec<TestToken>>,
    pub has_bailout: bool,
    pending_token_set: LexUnitSink,
    captured_tag_name: Option<String>,
    open_captured_tag_count: usize,
}

impl ParsingResult {
    pub fn new(input: &ChunkedInput, initial_mode_snapshot: TextParsingModeSnapshot) -> Self {
        let mut result = ParsingResult {
            token_sets: Vec::new(),
            has_bailout: false,
            pending_token_set: LexUnitSink::default(),
            captured_tag_name: None,
            open_captured_tag_count: 0,
        };

        // TODO
        result.has_bailout = result.parse(input, initial_mode_snapshot).is_err();

        result
    }

    fn add_pending_token_set(&mut self) {
        self.pending_token_set.flush();

        let tokens = self.pending_token_set.tokens.to_owned();

        if !tokens.is_empty() {
            self.token_sets.push(tokens);
        }

        self.pending_token_set = LexUnitSink::default();
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

            move |lex_unit: &LexUnit<'_>| {
                let mut result = result.borrow_mut();

                if let Some(Token::Eof) = lex_unit.get_token() {
                    result.add_pending_token_set();
                } else {
                    result
                        .pending_token_set
                        .add_lex_unit(lex_unit, mode_snapshot.get());
                }
            }
        };

        let tag_lex_unit_handler = {
            let result = Rc::clone(&result);
            let mode_snapshot = Rc::clone(&mode_snapshot);

            move |lex_unit: &LexUnit<'_>| {
                let mut result = result.borrow_mut();

                let captured_tag_name = result
                    .captured_tag_name
                    .to_owned()
                    .expect("Captured tag name should be set at this point");

                macro_rules! add_lex_unit {
                    ($lex_unit:ident) => {
                        result
                            .pending_token_set
                            .add_lex_unit($lex_unit, mode_snapshot.get());
                    };
                }

                match lex_unit.get_token() {
                    Some(Token::StartTag { ref name, .. })
                        if to_lower_string(name) == captured_tag_name =>
                    {
                        result.open_captured_tag_count += 1;

                        if result.open_captured_tag_count > 1 {
                            add_lex_unit!(lex_unit);
                        }

                        NextOutputType::LexUnit
                    }
                    Some(Token::EndTag { ref name, .. })
                        if to_lower_string(name) == captured_tag_name =>
                    {
                        result.open_captured_tag_count -= 1;

                        if result.open_captured_tag_count == 0 {
                            result.add_pending_token_set();

                            NextOutputType::TagPreview
                        } else {
                            add_lex_unit!(lex_unit);
                            NextOutputType::LexUnit
                        }
                    }
                    _ => {
                        add_lex_unit!(lex_unit);
                        NextOutputType::LexUnit
                    }
                }
            }
        };

        let tag_preview_handler = {
            let result = Rc::clone(&result);

            move |tag_preview: &TagPreview<'_>| match tag_preview {
                TagPreview::StartTag(name_info) => {
                    let mut result = result.borrow_mut();

                    result.captured_tag_name = Some(to_lower_string(name_info.get_name()));

                    NextOutputType::LexUnit
                }
                _ => NextOutputType::TagPreview,
            }
        };

        let mut transform_stream = TransformStream::new(
            BUFFER_SIZE,
            lex_unit_handler,
            tag_lex_unit_handler,
            tag_preview_handler,
        );

        transform_stream
            .get_tokenizer()
            .get_full_sm()
            .text_parsing_mode_change_handler = Some(Box::new({
            let mode_snapshot = Rc::clone(&mode_snapshot);

            move |s| mode_snapshot.set(s)
        }));

        input.parse(
            transform_stream,
            initial_mode_snapshot,
            NextOutputType::TagPreview,
        )
    }
}
