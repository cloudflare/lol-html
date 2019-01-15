#[macro_use]
mod actions;
mod conditions;

use crate::base::{Align, Chunk, Cursor, Range};

use crate::tokenizer::outputs::*;
use crate::tokenizer::state_machine::{
    ParsingLoopDirective, ParsingLoopTerminationReason, StateMachine, StateResult,
};
use crate::tokenizer::{FeedbackProviders, NextOutputType, TagName, TextType, TreeBuilderFeedback};
use failure::Error;
use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;

pub trait TagPreviewSink {
    fn handle_tag_preview(&mut self, tag_preview: &TagPreview<'_>) -> NextOutputType;
}

pub type State<S> = fn(&mut EagerStateMachine<S>, &Chunk<'_>) -> StateResult;

/// Eager state machine skips the majority of full state machine operations and, thus,
/// is faster. It also has much less requirements for buffering which makes it more
/// prone to bailouts caused by buffer exhaustion (actually it buffers only tag names).
///
/// Eager state machine produces tag previews as an output which serve as a hint for
/// the matcher which can then switch to the full state machine if required.
///
/// It's not guaranteed that tag preview will actually produce the token in the end
/// of the input (e.g. `<div` will produce a tag preview, but not tag token). However,
/// it's not a concern for our use case as no content will be erroneously captured
/// in this case.
pub struct EagerStateMachine<S: TagPreviewSink> {
    input_cursor: Cursor,
    tag_start: Option<usize>,
    ch_sequence_matching_start: Option<usize>,
    tag_name_start: usize,
    is_in_end_tag: bool,
    tag_name_hash: Option<u64>,
    last_start_tag_name_hash: Option<u64>,
    is_state_enter: bool,
    cdata_allowed: bool,
    tag_preview_sink: S,
    state: State<S>,
    closing_quote: u8,
    feedback_providers: Rc<RefCell<FeedbackProviders>>,
    pending_text_type_change: Option<TextType>,
    last_text_type: TextType,
}

impl<S: TagPreviewSink> EagerStateMachine<S> {
    pub fn new(tag_preview_sink: S, feedback_providers: Rc<RefCell<FeedbackProviders>>) -> Self {
        EagerStateMachine {
            input_cursor: Cursor::default(),
            tag_start: None,
            ch_sequence_matching_start: None,
            tag_name_start: 0,
            is_in_end_tag: false,
            tag_name_hash: None,
            last_start_tag_name_hash: None,
            is_state_enter: true,
            cdata_allowed: false,
            tag_preview_sink,
            state: EagerStateMachine::data_state,
            closing_quote: b'"',
            feedback_providers,
            pending_text_type_change: None,
            last_text_type: TextType::Data,
        }
    }

    fn create_tag_preview<'i>(&mut self, input: &'i Chunk<'i>) -> TagPreview<'i> {
        let name_range = Range {
            start: self.tag_name_start,
            end: self.input_cursor.pos(),
        };

        let tag_type = if self.is_in_end_tag {
            self.is_in_end_tag = false;
            TagType::EndTag
        } else {
            self.last_start_tag_name_hash = self.tag_name_hash;
            TagType::StartTag
        };

        TagPreview::new(input, tag_type, name_range, self.tag_name_hash)
    }

    fn get_feedback_for_tag(
        &mut self,
        tag_preview: &TagPreview<'_>,
    ) -> Result<TreeBuilderFeedback, Error> {
        let mut feedback_providers = self.feedback_providers.borrow_mut();
        let name_hash = tag_preview.name_hash();

        Ok(match tag_preview.tag_type() {
            TagType::StartTag => {
                feedback_providers
                    .ambiguity_guard
                    .track_start_tag(name_hash)?;

                feedback_providers
                    .tree_builder_simulator
                    .get_feedback_for_start_tag_name(name_hash)
            }
            TagType::EndTag => {
                feedback_providers.ambiguity_guard.track_end_tag(name_hash);

                feedback_providers
                    .tree_builder_simulator
                    .get_feedback_for_end_tag_name(name_hash)
            }
        })
    }

    fn handle_tree_builder_feedback(
        &mut self,
        feedback: TreeBuilderFeedback,
        tag_start: usize,
    ) -> ParsingLoopDirective {
        match feedback {
            TreeBuilderFeedback::SwitchTextType(text_type) => {
                // NOTE: we can't switch type immediately as we
                // are in the middle of tag parsing. So, we need
                // to switch later on the `emit_tag` action.
                self.pending_text_type_change = Some(text_type);
                ParsingLoopDirective::None
            }
            TreeBuilderFeedback::SetAllowCdata(cdata_allowed) => {
                self.cdata_allowed = cdata_allowed;
                ParsingLoopDirective::None
            }
            TreeBuilderFeedback::RequestLexeme(_) => ParsingLoopDirective::Break(
                ParsingLoopTerminationReason::LexemeRequiredForAdjustment(
                    self.create_bookmark(tag_start),
                ),
            ),
            TreeBuilderFeedback::None => ParsingLoopDirective::None,
        }
    }
}

impl<S: TagPreviewSink> StateMachine for EagerStateMachine<S> {
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
        // NOTE: if we are in character sequence matching we need
        // to block from the position where matching starts. We don't
        // need to do that manually in full state machine because it
        // always blocks all bytes starting from lexeme start and it's
        // guaranteed that character sequence matching occurs withih
        // lexeme boundaries.
        match (self.tag_start, self.ch_sequence_matching_start) {
            (Some(tag_start), Some(ch_sequence_matching_start)) => {
                input.len() - min(tag_start, ch_sequence_matching_start)
            }
            (Some(tag_start), None) => input.len() - tag_start,
            (None, Some(ch_sequence_matching_start)) => input.len() - ch_sequence_matching_start,
            (None, None) => 0,
        }
    }

    fn adjust_for_next_input(&mut self) {
        if let Some(tag_start) = self.tag_start {
            self.input_cursor.align(tag_start);
            self.tag_name_start.align(tag_start);
            self.tag_start = Some(0);
        } else {
            self.input_cursor = Cursor::default();
        }
    }

    #[inline]
    fn adjust_to_bookmark(&mut self, _pos: usize) {
        trace!(@noop);
    }

    #[inline]
    fn enter_ch_sequence_matching(&mut self) {
        self.ch_sequence_matching_start = Some(self.input_cursor.pos());
    }

    #[inline]
    fn leave_ch_sequence_matching(&mut self) {
        self.ch_sequence_matching_start = None;
    }
}
