#[macro_use]
mod actions;
mod conditions;

use crate::base::{Align, Chunk, Cursor, Range};
use crate::html::LocalNameHash;
use crate::parser::outputs::*;
use crate::parser::state_machine::{
    ParsingLoopDirective, ParsingLoopResult, StateMachine, StateMachineBookmark, StateResult,
};
use crate::parser::{ParserDirective, TextType, TreeBuilderFeedback, TreeBuilderSimulator};
use std::cell::RefCell;
use std::rc::Rc;

const DEFAULT_ATTR_BUFFER_CAPACITY: usize = 256;

pub trait LexemeSink {
    fn handle_tag(&mut self, lexeme: &TagLexeme<'_>) -> ParserDirective;
    fn handle_non_tag_content(&mut self, lexeme: &NonTagContentLexeme<'_>);
}

pub type State<S> = fn(&mut Lexer<S>, &Chunk<'_>) -> StateResult;
pub type SharedAttributeBuffer = Rc<RefCell<Vec<AttributeOultine>>>;

pub struct Lexer<S: LexemeSink> {
    input_cursor: Cursor,
    lexeme_start: usize,
    token_part_start: usize,
    is_state_enter: bool,
    cdata_allowed: bool,
    lexeme_sink: S,
    state: State<S>,
    current_tag_token: Option<TagTokenOutline>,
    current_non_tag_content_token: Option<NonTagContentTokenOutline>,
    current_attr: Option<AttributeOultine>,
    last_start_tag_name_hash: LocalNameHash,
    closing_quote: u8,
    attr_buffer: SharedAttributeBuffer,
    tree_builder_simulator: Rc<RefCell<TreeBuilderSimulator>>,
    last_text_type: TextType,
    should_silently_consume_current_tag_only: bool,
}

impl<S: LexemeSink> Lexer<S> {
    pub fn new(lexeme_sink: S, tree_builder_simulator: Rc<RefCell<TreeBuilderSimulator>>) -> Self {
        Lexer {
            input_cursor: Cursor::default(),
            lexeme_start: 0,
            token_part_start: 0,
            is_state_enter: true,
            cdata_allowed: false,
            lexeme_sink,
            state: Lexer::data_state,
            current_tag_token: None,
            current_non_tag_content_token: None,
            current_attr: None,
            last_start_tag_name_hash: LocalNameHash::empty(),
            closing_quote: b'"',
            attr_buffer: Rc::new(RefCell::new(Vec::with_capacity(
                DEFAULT_ATTR_BUFFER_CAPACITY,
            ))),
            tree_builder_simulator,
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

    fn handle_tree_builder_feedback(
        &mut self,
        feedback: TreeBuilderFeedback,
        lexeme: &TagLexeme<'_>,
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
            TreeBuilderFeedback::RequestLexeme(mut callback) => {
                let feedback = callback(&mut self.tree_builder_simulator.borrow_mut(), lexeme);

                self.handle_tree_builder_feedback(feedback, lexeme)
            }
            TreeBuilderFeedback::None => ParsingLoopDirective::None,
        }
    }

    #[inline]
    fn emit_lexeme(&mut self, lexeme: &NonTagContentLexeme<'_>) {
        trace!(@output lexeme);

        self.lexeme_start = lexeme.raw_range().end;
        self.lexeme_sink.handle_non_tag_content(lexeme);
    }

    #[inline]
    fn emit_tag_lexeme(&mut self, lexeme: &TagLexeme<'_>) -> ParserDirective {
        trace!(@output lexeme);

        self.lexeme_start = lexeme.raw_range().end;

        if self.should_silently_consume_current_tag_only {
            self.should_silently_consume_current_tag_only = false;
            ParserDirective::ScanForTags
        } else {
            self.lexeme_sink.handle_tag(lexeme)
        }
    }

    #[inline]
    fn create_lexeme_with_raw<'i, T>(
        &mut self,
        input: &'i Chunk<'i>,
        token: T,
        raw_end: usize,
    ) -> Lexeme<'i, T> {
        Lexeme::new(
            input,
            token,
            Range {
                start: self.lexeme_start,
                end: raw_end,
            },
        )
    }

    #[inline]
    fn create_lexeme_with_raw_inclusive<'i, T>(
        &mut self,
        input: &'i Chunk<'i>,
        token: T,
    ) -> Lexeme<'i, T> {
        let raw_end = self.input_cursor.pos() + 1;

        self.create_lexeme_with_raw(input, token, raw_end)
    }

    #[inline]
    fn create_lexeme_with_raw_exclusive<'i, T>(
        &mut self,
        input: &'i Chunk<'i>,
        token: T,
    ) -> Lexeme<'i, T> {
        let raw_end = self.input_cursor.pos();

        self.create_lexeme_with_raw(input, token, raw_end)
    }
}

impl<S: LexemeSink> StateMachine for Lexer<S> {
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
        self.current_tag_token.align(self.lexeme_start);
        self.current_non_tag_content_token.align(self.lexeme_start);
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
