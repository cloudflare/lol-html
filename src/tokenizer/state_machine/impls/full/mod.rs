#[macro_use]
mod actions;
mod conditions;

use crate::base::{Align, Chunk, Cursor, Range};
use crate::tokenizer::outputs::*;
use crate::tokenizer::state_machine::{
    ParsingLoopDirective, ParsingLoopResult, StateMachine, StateMachineBookmark, StateResult,
};
use crate::tokenizer::{
    FeedbackProviders, NextOutputType, ParsingLoopTerminationReason, TagName, TextType,
    TreeBuilderFeedback,
};
use failure::Error;
use std::cell::RefCell;
use std::rc::Rc;

const DEFAULT_ATTR_BUFFER_CAPACITY: usize = 256;

pub trait LexemeSink {
    fn handle_tag(&mut self, lexeme: &Lexeme<'_>) -> NextOutputType;
    fn handle_non_tag_content(&mut self, lexeme: &Lexeme<'_>);
}

pub type State<S> = fn(&mut FullStateMachine<S>, &Chunk<'_>) -> StateResult;

pub struct FullStateMachine<S: LexemeSink> {
    input_cursor: Cursor,
    lexeme_start: usize,
    token_part_start: usize,
    is_state_enter: bool,
    cdata_allowed: bool,
    lexeme_sink: S,
    state: State<S>,
    current_token: Option<TokenView>,
    current_attr: Option<AttributeView>,
    last_start_tag_name_hash: Option<u64>,
    closing_quote: u8,
    attr_buffer: Rc<RefCell<Vec<AttributeView>>>,
    feedback_providers: Rc<RefCell<FeedbackProviders>>,
    last_text_type: TextType,
    should_silently_consume_current_tag_only: bool,
}

impl<S: LexemeSink> FullStateMachine<S> {
    pub fn new(lexeme_sink: S, feedback_providers: Rc<RefCell<FeedbackProviders>>) -> Self {
        FullStateMachine {
            input_cursor: Cursor::default(),
            lexeme_start: 0,
            token_part_start: 0,
            is_state_enter: true,
            cdata_allowed: false,
            lexeme_sink,
            state: FullStateMachine::data_state,
            current_token: None,
            current_attr: None,
            last_start_tag_name_hash: None,
            closing_quote: b'"',
            attr_buffer: Rc::new(RefCell::new(Vec::with_capacity(
                DEFAULT_ATTR_BUFFER_CAPACITY,
            ))),
            feedback_providers,
            last_text_type: TextType::Data,
            should_silently_consume_current_tag_only: false,
        }
    }

    #[inline]
    pub fn silently_consume_current_tag_only(
        &mut self,
        input: &Chunk<'_>,
        bookmark: StateMachineBookmark,
    ) -> ParsingLoopResult {
        self.should_silently_consume_current_tag_only = true;

        self.continue_from_bookmark(input, bookmark)
    }

    fn get_feedback_for_tag(
        &mut self,
        token: &Option<TokenView>,
    ) -> Result<TreeBuilderFeedback, Error> {
        let mut feedback_providers = self.feedback_providers.borrow_mut();

        match *token {
            Some(TokenView::StartTag { name_hash, .. }) => {
                // NOTE: if we are silently parsing the tag to get tree builder
                // feedback for the eager state machine then guard check has been
                // already activated by the eager state machine.
                if !self.should_silently_consume_current_tag_only {
                    feedback_providers
                        .ambiguity_guard
                        .track_start_tag(name_hash)?;
                }

                Ok(feedback_providers
                    .tree_builder_simulator
                    .get_feedback_for_start_tag_name(name_hash))
            }
            Some(TokenView::EndTag { name_hash, .. }) => {
                // NOTE: if we are silently parsing the tag to get tree builder
                // feedback for the eager state machine then guard check has been
                // already activated by the eager state machine.
                if !self.should_silently_consume_current_tag_only {
                    feedback_providers.ambiguity_guard.track_end_tag(name_hash);
                }

                Ok(feedback_providers
                    .tree_builder_simulator
                    .get_feedback_for_end_tag_name(name_hash))
            }
            _ => unreachable!("Token should be a start or an end tag at this point"),
        }
    }

    fn handle_tree_builder_feedback(
        &mut self,
        feedback: TreeBuilderFeedback,
        lexeme: &Lexeme<'_>,
    ) -> ParsingLoopDirective {
        match feedback {
            TreeBuilderFeedback::SwitchTextType(text_type) => {
                self.switch_text_type(text_type);
                ParsingLoopDirective::Continue
            }
            TreeBuilderFeedback::SetAllowCdata(cdata_allowed) => {
                self.cdata_allowed = cdata_allowed;
                ParsingLoopDirective::None
            }
            TreeBuilderFeedback::RequestLexeme(callback) => {
                let feedback = {
                    let tree_builder_simulator =
                        &mut self.feedback_providers.borrow_mut().tree_builder_simulator;

                    callback(tree_builder_simulator, &lexeme)
                };

                self.handle_tree_builder_feedback(feedback, lexeme)
            }
            TreeBuilderFeedback::None => ParsingLoopDirective::None,
        }
    }

    #[inline]
    fn set_next_lexeme_start(&mut self, curr_lexeme: &Lexeme<'_>) {
        self.lexeme_start = curr_lexeme.raw_range().end;
    }

    #[inline]
    fn emit_lexeme(&mut self, lexeme: &Lexeme<'_>) {
        trace!(@output lexeme);

        self.set_next_lexeme_start(lexeme);
        self.lexeme_sink.handle_non_tag_content(lexeme);
    }

    #[inline]
    fn emit_tag_lexeme(&mut self, lexeme: &Lexeme<'_>) -> NextOutputType {
        trace!(@output lexeme);

        self.set_next_lexeme_start(lexeme);

        if self.should_silently_consume_current_tag_only {
            self.should_silently_consume_current_tag_only = false;
            NextOutputType::TagPreview
        } else {
            self.lexeme_sink.handle_tag(lexeme)
        }
    }

    #[inline]
    fn create_lexeme_with_raw<'i>(
        &mut self,
        input: &'i Chunk<'i>,
        token: Option<TokenView>,
        raw_end: usize,
    ) -> Lexeme<'i> {
        let raw_range = Range {
            start: self.lexeme_start,
            end: raw_end,
        };

        Lexeme::new(input, token, raw_range)
    }

    #[inline]
    fn create_lexeme_with_raw_inclusive<'i>(
        &mut self,
        input: &'i Chunk<'i>,
        token: Option<TokenView>,
    ) -> Lexeme<'i> {
        let raw_end = self.input_cursor.pos() + 1;

        self.create_lexeme_with_raw(input, token, raw_end)
    }

    #[inline]
    fn create_lexeme_with_raw_exclusive<'i>(
        &mut self,
        input: &'i Chunk<'i>,
        token: Option<TokenView>,
    ) -> Lexeme<'i> {
        let raw_end = self.input_cursor.pos();

        self.create_lexeme_with_raw(input, token, raw_end)
    }
}

impl<S: LexemeSink> StateMachine for FullStateMachine<S> {
    impl_common_sm_accessors!();

    #[inline]
    fn set_state(&mut self, state: State<S>) {
        self.state = state;
    }

    #[inline]
    fn state(&self) -> State<S> {
        self.state
    }

    #[inline]
    fn get_blocked_byte_count(&self, input: &Chunk<'_>) -> usize {
        input.len() - self.lexeme_start
    }

    fn adjust_for_next_input(&mut self) {
        self.input_cursor.align(self.lexeme_start);
        self.token_part_start.align(self.lexeme_start);
        self.current_token.align(self.lexeme_start);
        self.current_attr.align(self.lexeme_start);

        self.lexeme_start = 0;
    }

    #[inline]
    fn adjust_to_bookmark(&mut self, pos: usize) {
        self.lexeme_start = pos;
    }

    #[inline]
    fn enter_ch_sequence_matching(&mut self) {
        trace!(@noop);
    }

    #[inline]
    fn leave_ch_sequence_matching(&mut self) {
        trace!(@noop);
    }
}
