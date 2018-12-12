use crate::harness::tokenizer_test::{
    Bailout, ChunkedInput, LexUnitSink, TestCase, TestFixture, TestToken, BUFFER_SIZE,
};
use cool_thing::tokenizer::{LexUnit, NextOutputType, TagPreview, TextParsingModeSnapshot};
use cool_thing::transform_stream::TransformStream;
use failure::Error;
use itertools::izip;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct ParsingResult {
    pub bailout: Option<Bailout>,
    pub lex_unit_sink: LexUnitSink,
}

impl ParsingResult {
    pub fn new(input: &ChunkedInput, initial_mode_snapshot: TextParsingModeSnapshot) -> Self {
        let mut result = ParsingResult {
            bailout: None,
            lex_unit_sink: LexUnitSink::default(),
        };

        // TODO use bailout handler later with substitution and test eager state machine as well.
        if let Err(ref e) = result.parse(input, initial_mode_snapshot) {
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

            move |lex_unit: &LexUnit<'_>| {
                result
                    .borrow_mut()
                    .lex_unit_sink
                    .add_lex_unit(lex_unit, mode_snapshot.get());
            }
        };

        let tag_lex_unit_handler = {
            let result = Rc::clone(&result);
            let mode_snapshot = Rc::clone(&mode_snapshot);

            move |lex_unit: &LexUnit<'_>| {
                result
                    .borrow_mut()
                    .lex_unit_sink
                    .add_lex_unit(lex_unit, mode_snapshot.get());

                NextOutputType::LexUnit
            }
        };

        let tag_preview_handler = |_: &TagPreview<'_>| NextOutputType::TagPreview;

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
            NextOutputType::LexUnit,
        )
    }

    fn add_bailout(&mut self, reason: &Error) {
        self.bailout = Some(Bailout {
            reason: format!("{:?}", reason),
            parsed_chunk: self.get_cumulative_raw_string(),
        });
    }

    pub fn get_cumulative_raw_string(&self) -> String {
        String::from_utf8(
            self.lex_unit_sink
                .raw_slices
                .iter()
                .fold(Vec::new(), |mut c, s| {
                    c.extend_from_slice(s);
                    c
                }),
        )
        .unwrap()
    }

    pub fn into_token_raw_pairs(
        mut self,
    ) -> Option<Vec<(TestToken, String, TextParsingModeSnapshot)>> {
        // NOTE: remove EOF which doesn't have raw representation
        self.lex_unit_sink.tokens.pop();

        // NOTE: there are cases where text token can contain
        // part that is ignored during parsing, but still has raw
        // representation. E.g. `a</>b` will produce `ab` text
        // token, however we'll have `a` and `</>b` raw strings and,
        // thus, we can't produce one-on-one mapping.
        if self.lex_unit_sink.tokens.len() == self.lex_unit_sink.raw_slices.len() {
            Some(
                izip!(
                    self.lex_unit_sink.tokens.into_iter(),
                    self.lex_unit_sink
                        .raw_slices
                        .into_iter()
                        .map(|s| String::from_utf8(s).unwrap()),
                    self.lex_unit_sink.text_parsing_mode_snapshots.into_iter()
                )
                .collect(),
            )
        } else {
            None
        }
    }
}

/// Tests that full state machine produces correct lex units.
pub struct FullStateMachineTests;

impl FullStateMachineTests {
    fn assert_tokens_have_correct_raw_strings(actual: ParsingResult) {
        if let Some(token_raw_pairs) = actual.into_token_raw_pairs() {
            for (token, raw, text_parsing_mode_snapshot) in token_raw_pairs {
                let raw = raw.into();
                let actual = ParsingResult::new(&raw, text_parsing_mode_snapshot);

                assert_eql!(
                    actual.lex_unit_sink.tokens,
                    vec![token.to_owned(), TestToken::Eof],
                    raw,
                    text_parsing_mode_snapshot,
                    "Token's raw string doesn't produce the same token"
                );
            }
        }
    }
}

impl TestFixture for FullStateMachineTests {
    fn get_test_description_suffix() -> &'static str {
        "Full state machine"
    }

    fn run_test_case(test: &TestCase, initial_mode_snapshot: TextParsingModeSnapshot) {
        let actual = ParsingResult::new(&test.input, initial_mode_snapshot);

        assert_eql!(
            actual.bailout,
            test.expected_bailout,
            test.input,
            initial_mode_snapshot,
            "Tokenizer bailout error mismatch"
        );

        if actual.bailout.is_none() {
            assert_eql!(
                actual.lex_unit_sink.tokens,
                test.expected_tokens,
                test.input,
                initial_mode_snapshot,
                "Token mismatch"
            );

            assert_eql!(
                actual.get_cumulative_raw_string(),
                test.input,
                test.input,
                initial_mode_snapshot,
                "Cumulative raw strings mismatch"
            );

            Self::assert_tokens_have_correct_raw_strings(actual);
        }
    }
}

tokenizer_test_fixture!(FullStateMachineTests);
