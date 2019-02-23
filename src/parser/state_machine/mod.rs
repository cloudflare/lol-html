#[macro_use]
mod syntax_dsl;

#[macro_use]
mod syntax;

use crate::base::{Chunk, Cursor};
use crate::parser::{ParserDirective, TextType};
use failure::Error;

#[derive(Debug, Copy, Clone)]
pub struct StateMachineBookmark {
    pub cdata_allowed: bool,
    pub text_type: TextType,
    pub last_start_tag_name_hash: Option<u64>,
    pub pos: usize,
}

pub enum ParsingLoopTerminationReason {
    ParserDirectiveChange(ParserDirective, StateMachineBookmark),
    LexemeRequiredForAdjustment(StateMachineBookmark),
    EndOfInput { blocked_byte_count: usize },
}

pub enum ParsingLoopDirective {
    Break(ParsingLoopTerminationReason),
    Continue,
    None,
}

pub type StateResult = Result<ParsingLoopDirective, Error>;
pub type ParsingLoopResult = Result<ParsingLoopTerminationReason, Error>;

pub trait StateMachineActions {
    fn emit_eof(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn emit_text(&mut self, input: &Chunk<'_>, _ch: Option<u8>);
    fn emit_current_token(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn emit_tag(&mut self, input: &Chunk<'_>, ch: Option<u8>) -> StateResult;
    fn emit_current_token_and_eof(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn emit_raw_without_token(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn emit_raw_without_token_and_eof(&mut self, input: &Chunk<'_>, ch: Option<u8>);

    fn create_start_tag(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn create_end_tag(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn create_doctype(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn create_comment(&mut self, input: &Chunk<'_>, ch: Option<u8>);

    fn start_token_part(&mut self, input: &Chunk<'_>, ch: Option<u8>);

    fn mark_comment_text_end(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn shift_comment_text_end_by(&mut self, input: &Chunk<'_>, ch: Option<u8>, offset: usize);

    fn set_force_quirks(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn finish_doctype_name(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn finish_doctype_public_id(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn finish_doctype_system_id(&mut self, input: &Chunk<'_>, ch: Option<u8>);

    fn finish_tag_name(&mut self, input: &Chunk<'_>, ch: Option<u8>) -> StateResult;
    fn update_tag_name_hash(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn mark_as_self_closing(&mut self, input: &Chunk<'_>, ch: Option<u8>);

    fn start_attr(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn finish_attr_name(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn finish_attr_value(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn finish_attr(&mut self, input: &Chunk<'_>, ch: Option<u8>);

    fn set_closing_quote_to_double(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn set_closing_quote_to_single(&mut self, input: &Chunk<'_>, ch: Option<u8>);

    fn mark_tag_start(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn unmark_tag_start(&mut self, input: &Chunk<'_>, ch: Option<u8>);

    fn enter_cdata(&mut self, input: &Chunk<'_>, ch: Option<u8>);
    fn leave_cdata(&mut self, input: &Chunk<'_>, ch: Option<u8>);
}

pub trait StateMachineConditions {
    fn is_appropriate_end_tag(&self, ch: Option<u8>) -> bool;
    fn cdata_allowed(&self, ch: Option<u8>) -> bool;
}

pub trait StateMachine: StateMachineActions + StateMachineConditions {
    define_states!();

    fn state(&self) -> fn(&mut Self, &Chunk<'_>) -> StateResult;
    fn set_state(&mut self, state: fn(&mut Self, &Chunk<'_>) -> StateResult);

    fn input_cursor(&mut self) -> &mut Cursor;
    fn set_input_cursor(&mut self, input_cursor: Cursor);

    fn is_state_enter(&self) -> bool;
    fn set_is_state_enter(&mut self, val: bool);

    fn last_start_tag_name_hash(&self) -> Option<u64>;
    fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>);

    fn set_last_text_type(&mut self, text_type: TextType);
    fn last_text_type(&self) -> TextType;

    fn set_cdata_allowed(&mut self, cdata_allowed: bool);

    fn closing_quote(&self) -> u8;

    fn adjust_for_next_input(&mut self);
    fn adjust_to_bookmark(&mut self, pos: usize);
    fn enter_ch_sequence_matching(&mut self);
    fn leave_ch_sequence_matching(&mut self);
    fn get_blocked_byte_count(&self, input: &Chunk<'_>) -> usize;

    fn run_parsing_loop(&mut self, input: &Chunk<'_>) -> ParsingLoopResult {
        loop {
            let state = self.state();

            if let ParsingLoopDirective::Break(reason) = state(self, input)? {
                return Ok(reason);
            }
        }
    }

    fn continue_from_bookmark(
        &mut self,
        input: &Chunk<'_>,
        bookmark: StateMachineBookmark,
    ) -> ParsingLoopResult {
        self.set_cdata_allowed(bookmark.cdata_allowed);
        self.switch_text_type(bookmark.text_type);
        self.set_last_start_tag_name_hash(bookmark.last_start_tag_name_hash);
        self.adjust_to_bookmark(bookmark.pos);
        self.set_input_cursor(Cursor::new(bookmark.pos));

        self.run_parsing_loop(input)
    }

    #[inline]
    fn break_on_end_of_input(&mut self, input: &Chunk<'_>) -> StateResult {
        let blocked_byte_count = self.get_blocked_byte_count(input);

        if !input.is_last() {
            self.adjust_for_next_input()
        }

        Ok(ParsingLoopDirective::Break(
            ParsingLoopTerminationReason::EndOfInput { blocked_byte_count },
        ))
    }

    #[inline]
    fn create_bookmark(&self, pos: usize) -> StateMachineBookmark {
        StateMachineBookmark {
            cdata_allowed: self.cdata_allowed(None),
            text_type: self.last_text_type(),
            last_start_tag_name_hash: self.last_start_tag_name_hash(),
            pos,
        }
    }

    #[inline]
    fn change_parser_directive(
        &self,
        pos: usize,
        new_parser_directive: ParserDirective,
    ) -> ParsingLoopDirective {
        ParsingLoopDirective::Break(ParsingLoopTerminationReason::ParserDirectiveChange(
            new_parser_directive,
            self.create_bookmark(pos),
        ))
    }

    #[inline]
    fn switch_state(&mut self, state: fn(&mut Self, &Chunk<'_>) -> StateResult) {
        self.set_state(state);
        self.set_is_state_enter(true);
    }

    #[inline]
    fn switch_text_type(&mut self, text_type: TextType) {
        self.set_last_text_type(text_type);

        self.switch_state(match text_type {
            TextType::Data => Self::data_state,
            TextType::PlainText => Self::plaintext_state,
            TextType::RCData => Self::rcdata_state,
            TextType::RawText => Self::rawtext_state,
            TextType::ScriptData => Self::script_data_state,
            TextType::CDataSection => Self::cdata_section_state,
        });
    }
}

macro_rules! impl_common_sm_accessors {
    () => {
        #[inline]
        fn input_cursor(&mut self) -> &mut Cursor {
            &mut self.input_cursor
        }

        #[inline]
        fn set_input_cursor(&mut self, input_cursor: Cursor) {
            self.input_cursor = input_cursor;
        }

        #[inline]
        fn is_state_enter(&self) -> bool {
            self.is_state_enter
        }

        #[inline]
        fn set_is_state_enter(&mut self, val: bool) {
            self.is_state_enter = val;
        }

        #[inline]
        fn set_last_text_type(&mut self, text_type: TextType) {
            self.last_text_type = text_type;
        }

        #[inline]
        fn last_text_type(&self) -> TextType {
            self.last_text_type
        }

        #[inline]
        fn closing_quote(&self) -> u8 {
            self.closing_quote
        }

        #[inline]
        fn last_start_tag_name_hash(&self) -> Option<u64> {
            self.last_start_tag_name_hash
        }

        #[inline]
        fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
            self.last_start_tag_name_hash = name_hash;
        }

        #[inline]
        fn set_cdata_allowed(&mut self, cdata_allowed: bool) {
            self.cdata_allowed = cdata_allowed;
        }
    };
}

macro_rules! impl_common_sm_actions {
    () => {
        #[inline]
        fn set_closing_quote_to_double(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
            self.closing_quote = b'"';
        }

        #[inline]
        fn set_closing_quote_to_single(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
            self.closing_quote = b'\'';
        }

        #[inline]
        fn enter_cdata(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
            self.set_last_text_type(TextType::CDataSection);
        }

        #[inline]
        fn leave_cdata(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
            self.set_last_text_type(TextType::Data);
        }
    };
}

macro_rules! noop_action {
    ($($fn_name:ident),*) => {
        $(
            #[inline]
            fn $fn_name(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
                trace!(@noop);
            }
        )*
    };
}
