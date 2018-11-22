#[macro_use]
mod actions;
mod conditions;

use base::{Align, Chunk, Cursor, Range};
use std::cell::RefCell;
use std::rc::Rc;
use tokenizer::outputs::*;
use tokenizer::state_machine::{ParsingLoopTerminationReason, StateMachine, StateResult};
use tokenizer::tree_builder_simulator::*;
use tokenizer::{NextOutputType, TagName, TagPreviewHandler, TextParsingMode};

pub type State<H> = fn(&mut EagerStateMachine<H>, &Chunk) -> StateResult;

pub struct EagerStateMachine<H: TagPreviewHandler> {
    input_cursor: Cursor,
    tag_start: Option<usize>,
    tag_name_start: usize,
    is_in_end_tag: bool,
    tag_name_hash: Option<u64>,
    last_start_tag_name_hash: Option<u64>,
    state_enter: bool,
    allow_cdata: bool,
    tag_preview_handler: H,
    state: State<H>,
    closing_quote: u8,
    tree_builder_simulator: Rc<RefCell<TreeBuilderSimulator>>,
    last_text_parsing_mode_change: TextParsingMode,
}

impl<H: TagPreviewHandler> EagerStateMachine<H> {
    pub fn new(
        tag_preview_handler: H,
        tree_builder_simulator: &Rc<RefCell<TreeBuilderSimulator>>,
    ) -> Self {
        EagerStateMachine {
            input_cursor: Cursor::default(),
            tag_start: None,
            tag_name_start: 0,
            is_in_end_tag: false,
            tag_name_hash: None,
            last_start_tag_name_hash: None,
            state_enter: true,
            allow_cdata: false,
            tag_preview_handler,
            state: EagerStateMachine::data_state,
            closing_quote: b'"',
            tree_builder_simulator: Rc::clone(tree_builder_simulator),
            last_text_parsing_mode_change: TextParsingMode::Data,
        }
    }
}

impl<H: TagPreviewHandler> StateMachine for EagerStateMachine<H> {
    impl_common_sm_accessors!();

    #[inline]
    fn set_state(&mut self, state: State<H>) {
        self.state = state;
    }

    #[inline]
    fn get_state(&self) -> State<H> {
        self.state
    }

    #[inline]
    fn get_blocked_byte_count(&self, input: &Chunk) -> usize {
        if let Some(tag_start) = self.tag_start {
            input.len() - tag_start
        } else {
            0
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
        // Noop
    }

    #[inline]
    fn store_last_text_parsing_mode_change(&mut self, mode: TextParsingMode) {
        self.last_text_parsing_mode_change = mode;
    }
}
