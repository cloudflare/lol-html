#[macro_use]
mod actions;
mod conditions;

use base::{Align, Chunk, Cursor, Range};
use std::cell::RefCell;
use std::rc::Rc;
use tokenizer::outputs::*;
use tokenizer::tree_builder_simulator::*;
use tokenizer::{
    LexUnitHandler, ParsingLoopDirective, StateMachine, StateResult, TagLexUnitHandler,
    TagLexUnitResponse, TagName, TextParsingMode,
};

#[cfg(feature = "testing_api")]
use tokenizer::{TextParsingModeChangeHandler, TextParsingModeSnapshot};

const DEFAULT_ATTR_BUFFER_CAPACITY: usize = 256;

pub type FullStateMachineState<LH, TH> = fn(&mut FullStateMachine<LH, TH>, &Chunk) -> StateResult;

pub struct FullStateMachine<LH, TH>
where
    LH: LexUnitHandler,
    TH: TagLexUnitHandler,
{
    input_cursor: Cursor,
    lex_unit_start: usize,
    token_part_start: usize,
    state_enter: bool,
    allow_cdata: bool,
    lex_unit_handler: LH,
    tag_lex_unit_handler: TH,
    state: FullStateMachineState<LH, TH>,
    current_token: Option<TokenView>,
    current_attr: Option<AttributeView>,
    last_start_tag_name_hash: Option<u64>,
    closing_quote: u8,
    attr_buffer: Rc<RefCell<Vec<AttributeView>>>,
    tree_builder_simulator: TreeBuilderSimulator,

    #[cfg(feature = "testing_api")]
    text_parsing_mode_change_handler: Option<Box<dyn TextParsingModeChangeHandler>>,
}

impl<LH, TH> FullStateMachine<LH, TH>
where
    LH: LexUnitHandler,
    TH: TagLexUnitHandler,
{
    pub fn new(lex_unit_handler: LH, tag_lex_unit_handler: TH) -> Self {
        FullStateMachine {
            input_cursor: Cursor::default(),
            lex_unit_start: 0,
            token_part_start: 0,
            state_enter: true,
            allow_cdata: false,
            lex_unit_handler,
            tag_lex_unit_handler,
            state: FullStateMachine::data_state,
            current_token: None,
            current_attr: None,
            last_start_tag_name_hash: None,
            closing_quote: b'"',
            attr_buffer: Rc::new(RefCell::new(Vec::with_capacity(
                DEFAULT_ATTR_BUFFER_CAPACITY,
            ))),
            tree_builder_simulator: TreeBuilderSimulator::default(),

            #[cfg(feature = "testing_api")]
            text_parsing_mode_change_handler: None,
        }
    }

    fn handle_tree_builder_feedback(
        &mut self,
        feedback: TreeBuilderFeedback,
        lex_unit: &LexUnit,
    ) -> Option<ParsingLoopDirective> {
        match feedback {
            TreeBuilderFeedback::SwitchTextParsingMode(mode) => {
                notify_text_parsing_mode_change!(self, mode);
                self.set_text_parsing_mode(mode);
                Some(ParsingLoopDirective::Continue)
            }
            TreeBuilderFeedback::SetAllowCdata(allow_cdata) => {
                self.allow_cdata = allow_cdata;
                None
            }
            TreeBuilderFeedback::RequestLexUnit(callback) => {
                let feedback = callback(&mut self.tree_builder_simulator, &lex_unit);

                self.handle_tree_builder_feedback(feedback, lex_unit)
            }
            TreeBuilderFeedback::None => None,
        }
    }

    #[inline]
    fn emit_lex_unit(&mut self, input: &Chunk, token: Option<TokenView>, raw_range: Option<Range>) {
        let lex_unit = LexUnit::new(input, token, raw_range);

        self.lex_unit_handler.handle(&lex_unit);
    }

    #[inline]
    fn emit_lex_unit_with_raw(&mut self, input: &Chunk, token: Option<TokenView>, raw_end: usize) {
        let raw_range = Some(Range {
            start: self.lex_unit_start,
            end: raw_end,
        });

        self.lex_unit_start = raw_end;

        self.emit_lex_unit(input, token, raw_range);
    }

    #[inline]
    fn emit_lex_unit_with_raw_inclusive(&mut self, input: &Chunk, token: Option<TokenView>) {
        let raw_end = self.input_cursor.pos() + 1;

        self.emit_lex_unit_with_raw(input, token, raw_end);
    }

    #[inline]
    fn emit_lex_unit_with_raw_exclusive(&mut self, input: &Chunk, token: Option<TokenView>) {
        let raw_end = self.input_cursor.pos();

        self.emit_lex_unit_with_raw(input, token, raw_end);
    }

    #[inline]
    fn emit_tag_lex_unit<'c>(
        &mut self,
        input: &'c Chunk,
        token: Option<TokenView>,
    ) -> (LexUnit<'c>, TagLexUnitResponse) {
        let raw_end = self.input_cursor.pos() + 1;

        let raw_range = Some(Range {
            start: self.lex_unit_start,
            end: raw_end,
        });

        let lex_unit = LexUnit::new(input, token, raw_range);

        self.lex_unit_start = raw_end;

        let response = self.tag_lex_unit_handler.handle(&lex_unit);

        (lex_unit, response)
    }
}

impl<LH, TH> StateMachine for FullStateMachine<LH, TH>
where
    LH: LexUnitHandler,
    TH: TagLexUnitHandler,
{
    #[inline]
    fn set_state(&mut self, state: FullStateMachineState<LH, TH>) {
        self.state = state;
    }

    #[inline]
    fn get_state(&self) -> FullStateMachineState<LH, TH> {
        self.state
    }

    #[inline]
    fn get_input_cursor(&mut self) -> &mut Cursor {
        &mut self.input_cursor
    }

    #[inline]
    fn get_blocked_byte_count(&self, input: &Chunk) -> usize {
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
    fn set_is_state_enter(&mut self, val: bool) {
        self.state_enter = val;
    }

    #[inline]
    fn is_state_enter(&self) -> bool {
        self.state_enter
    }

    #[inline]
    fn get_closing_quote(&self) -> u8 {
        self.closing_quote
    }

    #[cfg(feature = "testing_api")]
    fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        self.last_start_tag_name_hash = name_hash;
    }

    #[cfg(feature = "testing_api")]
    fn set_text_parsing_mode_change_handler(
        &mut self,
        handler: Box<dyn TextParsingModeChangeHandler>,
    ) {
        self.text_parsing_mode_change_handler = Some(handler);
    }
}
