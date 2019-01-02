#[macro_use]
mod actions;
mod conditions;

use crate::base::{Align, Chunk, Cursor, Range};
use crate::tokenizer::outputs::*;
use crate::tokenizer::state_machine::{
    ParsingLoopDirective, ParsingLoopResult, StateMachine, StateMachineBookmark, StateResult,
};
use crate::tokenizer::{
    FeedbackProviders, NextOutputType, ParsingLoopTerminationReason, TagName, TextParsingMode,
    TreeBuilderFeedback,
};
use cfg_if::cfg_if;
use failure::Error;
use std::cell::RefCell;
use std::rc::Rc;

cfg_if! {
    if #[cfg(feature = "testing_api")] {
        use crate::tokenizer::{TextParsingModeChangeHandler, TextParsingModeSnapshot};
        use super::common::SharedTagConfirmationHandler;
    }
}

const DEFAULT_ATTR_BUFFER_CAPACITY: usize = 256;

pub trait LexUnitSink {
    fn handle_tag(&mut self, lex_unit: &LexUnit<'_>) -> NextOutputType;
    fn handle_non_tag_content(&mut self, lex_unit: &LexUnit<'_>);
}

pub type State<S> = fn(&mut FullStateMachine<S>, &Chunk<'_>) -> StateResult;

pub struct FullStateMachine<S: LexUnitSink> {
    input_cursor: Cursor,
    lex_unit_start: usize,
    token_part_start: usize,
    is_state_enter: bool,
    cdata_allowed: bool,
    lex_unit_sink: S,
    state: State<S>,
    current_token: Option<TokenView>,
    current_attr: Option<AttributeView>,
    last_start_tag_name_hash: Option<u64>,
    closing_quote: u8,
    attr_buffer: Rc<RefCell<Vec<AttributeView>>>,
    feedback_providers: Rc<RefCell<FeedbackProviders>>,
    last_text_parsing_mode_change: TextParsingMode,
    should_silently_consume_current_tag_only: bool,

    #[cfg(feature = "testing_api")]
    pub text_parsing_mode_change_handler: Option<Box<dyn TextParsingModeChangeHandler>>,

    #[cfg(feature = "testing_api")]
    pub tag_confirmation_handler: Option<SharedTagConfirmationHandler>,
}

impl<S: LexUnitSink> FullStateMachine<S> {
    pub fn new(lex_unit_sink: S, feedback_providers: Rc<RefCell<FeedbackProviders>>) -> Self {
        FullStateMachine {
            input_cursor: Cursor::default(),
            lex_unit_start: 0,
            token_part_start: 0,
            is_state_enter: true,
            cdata_allowed: false,
            lex_unit_sink,
            state: FullStateMachine::data_state,
            current_token: None,
            current_attr: None,
            last_start_tag_name_hash: None,
            closing_quote: b'"',
            attr_buffer: Rc::new(RefCell::new(Vec::with_capacity(
                DEFAULT_ATTR_BUFFER_CAPACITY,
            ))),
            feedback_providers,
            last_text_parsing_mode_change: TextParsingMode::Data,
            should_silently_consume_current_tag_only: false,

            #[cfg(feature = "testing_api")]
            text_parsing_mode_change_handler: None,

            #[cfg(feature = "testing_api")]
            tag_confirmation_handler: None,
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
        lex_unit: &LexUnit<'_>,
    ) -> ParsingLoopDirective {
        match feedback {
            TreeBuilderFeedback::SwitchTextParsingMode(mode) => {
                self.switch_text_parsing_mode(mode);
                ParsingLoopDirective::Continue
            }
            TreeBuilderFeedback::SetAllowCdata(cdata_allowed) => {
                self.cdata_allowed = cdata_allowed;
                ParsingLoopDirective::None
            }
            TreeBuilderFeedback::RequestLexUnit(callback) => {
                let feedback = {
                    let tree_builder_simulator =
                        &mut self.feedback_providers.borrow_mut().tree_builder_simulator;

                    callback(tree_builder_simulator, &lex_unit)
                };

                self.handle_tree_builder_feedback(feedback, lex_unit)
            }
            TreeBuilderFeedback::None => ParsingLoopDirective::None,
        }
    }

    #[inline]
    fn set_next_lex_unit_start(&mut self, curr_lex_unit: &LexUnit<'_>) {
        self.lex_unit_start = curr_lex_unit.raw_range().end;
    }

    #[inline]
    fn emit_lex_unit(&mut self, lex_unit: &LexUnit<'_>) {
        trace!(@lex_unit lex_unit);

        self.set_next_lex_unit_start(lex_unit);
        self.lex_unit_sink.handle_non_tag_content(lex_unit);
    }

    #[inline]
    fn emit_tag_lex_unit(&mut self, lex_unit: &LexUnit<'_>) -> NextOutputType {
        trace!(@lex_unit lex_unit);

        self.set_next_lex_unit_start(lex_unit);

        if self.should_silently_consume_current_tag_only {
            confirm_tag!(self);
            self.should_silently_consume_current_tag_only = false;
            NextOutputType::TagPreview
        } else {
            self.lex_unit_sink.handle_tag(lex_unit)
        }
    }

    #[inline]
    fn create_lex_unit_with_raw<'i>(
        &mut self,
        input: &'i Chunk<'i>,
        token: Option<TokenView>,
        raw_end: usize,
    ) -> LexUnit<'i> {
        let raw_range = Range {
            start: self.lex_unit_start,
            end: raw_end,
        };

        LexUnit::new(input, token, raw_range)
    }

    #[inline]
    fn create_lex_unit_with_raw_inclusive<'i>(
        &mut self,
        input: &'i Chunk<'i>,
        token: Option<TokenView>,
    ) -> LexUnit<'i> {
        let raw_end = self.input_cursor.pos() + 1;

        self.create_lex_unit_with_raw(input, token, raw_end)
    }

    #[inline]
    fn create_lex_unit_with_raw_exclusive<'i>(
        &mut self,
        input: &'i Chunk<'i>,
        token: Option<TokenView>,
    ) -> LexUnit<'i> {
        let raw_end = self.input_cursor.pos();

        self.create_lex_unit_with_raw(input, token, raw_end)
    }
}

impl<S: LexUnitSink> StateMachine for FullStateMachine<S> {
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
        input.len() - self.lex_unit_start
    }

    fn adjust_for_next_input(&mut self) {
        self.input_cursor.align(self.lex_unit_start);
        self.token_part_start.align(self.lex_unit_start);
        self.current_token.align(self.lex_unit_start);
        self.current_attr.align(self.lex_unit_start);

        self.lex_unit_start = 0;
    }

    #[inline]
    fn adjust_to_bookmark(&mut self, pos: usize) {
        self.lex_unit_start = pos;
    }

    #[inline]
    fn set_last_text_parsing_mode(&mut self, mode: TextParsingMode) {
        self.last_text_parsing_mode_change = mode;

        #[cfg(feature = "testing_api")]
        {
            if let Some(ref mut text_parsing_mode_change_handler) =
                self.text_parsing_mode_change_handler
            {
                let snapshot = TextParsingModeSnapshot {
                    mode,
                    last_start_tag_name_hash: self.last_start_tag_name_hash,
                };

                text_parsing_mode_change_handler.handle(snapshot);
            }
        }
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
