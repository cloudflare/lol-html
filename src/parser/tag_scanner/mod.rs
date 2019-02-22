#[macro_use]
mod actions;
mod conditions;

use crate::base::{Align, Chunk, Cursor, Range};

use crate::parser::outputs::*;
use crate::parser::state_machine::{
    ParsingLoopDirective, ParsingLoopTerminationReason, StateMachine, StateResult,
};
use crate::parser::{
    ParserDirective, TagName, TextType, TreeBuilderFeedback, TreeBuilderSimulator,
};
use failure::Error;
use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;

pub trait TagHintSink {
    fn handle_tag_hint(&mut self, tag_hint: &TagHint<'_>) -> ParserDirective;
}

pub type State<S> = fn(&mut TagScanner<S>, &Chunk<'_>) -> StateResult;

// Tag scanner skips the majority of lexer operations and, thus,
// is faster. It also has much less requirements for buffering which makes it more
// prone to bailouts caused by buffer exhaustion (actually it buffers only tag names).
//
// Tag scanner produces tag previews as an output which serve as a hint for
// the matcher which can then switch to the lexer if required.
//
// It's not guaranteed that tag preview will actually produce the token in the end
// of the input (e.g. `<div` will produce a tag preview, but not tag token). However,
// it's not a concern for our use case as no content will be erroneously captured
// in this case.
pub struct TagScanner<S: TagHintSink> {
    input_cursor: Cursor,
    tag_start: Option<usize>,
    ch_sequence_matching_start: Option<usize>,
    tag_name_start: usize,
    is_in_end_tag: bool,
    tag_name_hash: Option<u64>,
    last_start_tag_name_hash: Option<u64>,
    is_state_enter: bool,
    cdata_allowed: bool,
    tag_hint_sink: S,
    state: State<S>,
    closing_quote: u8,
    tree_builder_simulator: Rc<RefCell<TreeBuilderSimulator>>,
    pending_text_type_change: Option<TextType>,
    last_text_type: TextType,
}

impl<S: TagHintSink> TagScanner<S> {
    pub fn new(
        tag_hint_sink: S,
        tree_builder_simulator: Rc<RefCell<TreeBuilderSimulator>>,
    ) -> Self {
        TagScanner {
            input_cursor: Cursor::default(),
            tag_start: None,
            ch_sequence_matching_start: None,
            tag_name_start: 0,
            is_in_end_tag: false,
            tag_name_hash: None,
            last_start_tag_name_hash: None,
            is_state_enter: true,
            cdata_allowed: false,
            tag_hint_sink,
            state: TagScanner::data_state,
            closing_quote: b'"',
            tree_builder_simulator,
            pending_text_type_change: None,
            last_text_type: TextType::Data,
        }
    }

    fn create_tag_hint<'i>(&mut self, input: &'i Chunk<'i>) -> TagHint<'i> {
        let name_range = Range {
            start: self.tag_name_start,
            end: self.input_cursor.pos(),
        };

        let name_info = TagNameInfo::new(input, name_range, self.tag_name_hash);

        if self.is_in_end_tag {
            self.is_in_end_tag = false;

            TagHint::EndTag(name_info)
        } else {
            self.last_start_tag_name_hash = self.tag_name_hash;

            TagHint::StartTag(name_info)
        }
    }

    // TODO FeedbackProvider -> FeedbackProvider, get feedback for start tag and end tag
    // Get rid of tag hint
    // Separate lexemes for start and end tag

    fn get_feedback_for_tag(
        &mut self,
        tag_hint: &TagHint<'_>,
    ) -> Result<TreeBuilderFeedback, Error> {
        let mut tree_builder_simulator = self.tree_builder_simulator.borrow_mut();

        match tag_hint {
            TagHint::StartTag(name_info) => {
                let name_hash = name_info.name_hash();

                tree_builder_simulator.get_feedback_for_start_tag(name_hash, true)
            }
            TagHint::EndTag(name_info) => {
                let name_hash = name_info.name_hash();

                Ok(tree_builder_simulator.get_feedback_for_end_tag(name_hash, true))
            }
        }
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

impl<S: TagHintSink> StateMachine for TagScanner<S> {
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
        // need to do that manually in the lexer because it
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
