use crate::harness::tokenizer_test::{
    ChunkedInput, LexUnitSink, TestCase, TestFixture, TestToken, BUFFER_SIZE,
};
use cool_thing::base::Bytes;
use cool_thing::rewriting::Token;
use cool_thing::tokenizer::{
    LexUnit, NextOutputType, TagPreview, TextParsingModeSnapshot, TokenView,
};
use cool_thing::transform_stream::TransformStream;
use cool_thing::Error;
use encoding_rs::UTF_8;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn to_lower_string(bytes: &Bytes<'_>) -> String {
    let mut string = bytes.as_string(UTF_8);

    string.make_ascii_lowercase();

    string
}

fn get_descendants_of_top_level_elements(tokens: &[TestToken]) -> Vec<Vec<TestToken>> {
    tokens
        .to_owned()
        .into_iter()
        .fold(
            (Vec::new(), Vec::new(), None, 0),
            |(
                mut captures,
                mut pending_token_set,
                captured_tag_name,
                mut open_captured_tag_count,
            ): (Vec<Vec<_>>, Vec<_>, Option<_>, usize),
             t| {
                macro_rules! add_pending_token_set {
                    () => {
                        if !pending_token_set.is_empty() {
                            captures.push(pending_token_set);
                            pending_token_set = Vec::new();
                        }
                    };
                }

                let captured_tag_name = match captured_tag_name {
                    Some(captured_tag_name) => match t {
                        TestToken::StartTag { ref name, .. } if *name == captured_tag_name => {
                            open_captured_tag_count += 1;
                            pending_token_set.push(t.to_owned());

                            Some(captured_tag_name)
                        }
                        TestToken::EndTag { ref name, .. } if *name == captured_tag_name => {
                            open_captured_tag_count -= 1;

                            if open_captured_tag_count == 0 {
                                add_pending_token_set!();
                                None
                            } else {
                                pending_token_set.push(t.to_owned());
                                Some(captured_tag_name)
                            }
                        }
                        TestToken::Eof => {
                            add_pending_token_set!();

                            None
                        }
                        _ => {
                            pending_token_set.push(t.to_owned());
                            Some(captured_tag_name)
                        }
                    },
                    None => match t {
                        TestToken::StartTag { name, .. } => {
                            open_captured_tag_count = 1;

                            Some(name.to_owned())
                        }
                        _ => None,
                    },
                };

                (
                    captures,
                    pending_token_set,
                    captured_tag_name,
                    open_captured_tag_count,
                )
            },
        )
        .0
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

                if let Some(TokenView::Eof) = lex_unit.token_view() {
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

                match Token::try_from(lex_unit).as_ref() {
                    Some(Token::StartTag(t)) if to_lower_string(t.name()) == captured_tag_name => {
                        result.open_captured_tag_count += 1;

                        if result.open_captured_tag_count > 1 {
                            add_lex_unit!(lex_unit);
                        }

                        NextOutputType::LexUnit
                    }
                    Some(Token::EndTag(t)) if to_lower_string(t.name()) == captured_tag_name => {
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

                    result.captured_tag_name = Some(to_lower_string(name_info.name()));

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
            .tokenizer()
            .full_sm()
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

/// Tests that tokenizer correctly captures lex units that
/// are descendants of the top level elements.
pub struct TokenCapturingTests;

impl TestFixture for TokenCapturingTests {
    fn get_test_description_suffix() -> &'static str {
        "Content capturing"
    }

    fn run_test_case(test: &TestCase, initial_mode_snapshot: TextParsingModeSnapshot) {
        let actual = ParsingResult::new(&test.input, initial_mode_snapshot);
        let expected_token_sets = get_descendants_of_top_level_elements(&test.expected_tokens);

        if !actual.has_bailout {
            assert_eql!(
                actual.token_sets,
                expected_token_sets,
                test.input,
                initial_mode_snapshot,
                "Token sets mismatch"
            );
        }
    }
}

tokenizer_test_fixture!(TokenCapturingTests);
