use cool_thing::tokenizer::{LexUnit, NextOutputType, TagPreview, TextParsingModeSnapshot};
use cool_thing::transform_stream::TransformStream;
use cool_thing::Error;
use harness::tokenizer_test::{
    get_tag_tokens, ChunkedInput, TestCase, TestFixture, TestTagPreview, TestToken, BUFFER_SIZE,
};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ParsingResult {
    pub previews: Vec<TestTagPreview>,
    pub tokens_from_preview: Vec<TestToken>,
    pub has_bailout: bool,
    pending_tag_preview: Option<TestTagPreview>,
}

impl ParsingResult {
    pub fn new(input: &ChunkedInput, initial_mode_snapshot: TextParsingModeSnapshot) -> Self {
        let mut result = ParsingResult {
            previews: Vec::new(),
            tokens_from_preview: Vec::new(),
            has_bailout: false,
            pending_tag_preview: None,
        };

        // TODO
        result.has_bailout = result.parse(input, initial_mode_snapshot).is_err();

        result
    }

    fn parse(
        &mut self,
        input: &ChunkedInput,
        initial_mode_snapshot: TextParsingModeSnapshot,
    ) -> Result<(), Error> {
        let result = Rc::new(RefCell::new(self));

        let lex_unit_handler = |_: &LexUnit<'_>| {};

        let tag_lex_unit_handler = {
            let result = Rc::clone(&result);

            move |lex_unit: &LexUnit<'_>| {
                result.borrow_mut().add_lex_unit(lex_unit);

                NextOutputType::TagPreview
            }
        };

        let tag_preview_handler = {
            let result = Rc::clone(&result);

            move |tag_preview: &TagPreview<'_>| {
                result.borrow_mut().add_tag_preview(tag_preview);

                NextOutputType::LexUnit
            }
        };

        let transform_stream = TransformStream::new(
            BUFFER_SIZE,
            lex_unit_handler,
            tag_lex_unit_handler,
            tag_preview_handler,
        );

        input.parse(
            transform_stream,
            initial_mode_snapshot,
            NextOutputType::TagPreview,
        )
    }

    fn add_lex_unit(&mut self, lex_unit: &LexUnit<'_>) {
        self.tokens_from_preview.push(TestToken::new(
            lex_unit.get_token().expect("Tag should have a token"),
            lex_unit,
        ));

        let pending_preview = self
            .pending_tag_preview
            .take()
            .expect("Tag should have a preview");

        self.previews.push(pending_preview);
    }

    fn add_tag_preview(&mut self, tag_preview: &TagPreview<'_>) {
        // NOTE: it's not guaranteed that tag preview will produce
        // a tag at the end on input, it just gives matcher a hint
        // that there might be one (e.g. `<div` will not produce a
        // tag token). So we don't add tag preview unless we see
        // a token for it.
        self.pending_tag_preview = Some(TestTagPreview::new(tag_preview));
    }
}

/// Tests switching between state machines by parsing tags by both
/// of them in the context of the same tokenizer run.
pub struct StateMachineSwitchTests;

impl TestFixture for StateMachineSwitchTests {
    fn get_test_description_suffix() -> &'static str {
        "State machine switch"
    }

    fn run_test_case(test: &TestCase, initial_mode_snapshot: TextParsingModeSnapshot) {
        let actual = ParsingResult::new(&test.input, initial_mode_snapshot);
        let expected_tokens = get_tag_tokens(&test.expected_tokens);

        if !actual.has_bailout {
            assert_eql!(
                actual.previews,
                expected_tokens,
                test.input,
                initial_mode_snapshot,
                "Previews and tokens mismatch"
            );

            assert_eql!(
                actual.tokens_from_preview,
                expected_tokens,
                test.input,
                initial_mode_snapshot,
                "Tokens from preview mismatch"
            );
        }
    }
}

tokenizer_test_fixture!(StateMachineSwitchTests);
