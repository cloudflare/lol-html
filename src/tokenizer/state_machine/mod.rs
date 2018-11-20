#[macro_use]
mod syntax_dsl;

#[macro_use]
mod syntax;

mod impls;

pub use self::impls::*;

use base::{Chunk, Cursor};
use crate::Error;
use tokenizer::TextParsingMode;

#[cfg(feature = "testing_api")]
use tokenizer::TextParsingModeChangeHandler;

pub enum ParsingLoopTerminationReason {
    OutputTypeSwitch,
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
    fn emit_eof(&mut self, input: &Chunk, ch: Option<u8>);
    fn emit_chars(&mut self, input: &Chunk, _ch: Option<u8>);
    fn emit_current_token(&mut self, input: &Chunk, ch: Option<u8>);

    fn emit_tag(&mut self, input: &Chunk, ch: Option<u8>) -> StateResult;

    fn emit_current_token_and_eof(&mut self, input: &Chunk, ch: Option<u8>);
    fn emit_raw_without_token(&mut self, input: &Chunk, ch: Option<u8>);
    fn emit_raw_without_token_and_eof(&mut self, input: &Chunk, ch: Option<u8>);

    fn create_start_tag(&mut self, input: &Chunk, ch: Option<u8>);
    fn create_end_tag(&mut self, input: &Chunk, ch: Option<u8>);
    fn create_doctype(&mut self, input: &Chunk, ch: Option<u8>);
    fn create_comment(&mut self, input: &Chunk, ch: Option<u8>);

    fn start_token_part(&mut self, input: &Chunk, ch: Option<u8>);

    fn mark_comment_text_end(&mut self, input: &Chunk, ch: Option<u8>);
    fn shift_comment_text_end_by(&mut self, input: &Chunk, ch: Option<u8>, offset: usize);

    fn set_force_quirks(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_doctype_name(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_doctype_public_id(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_doctype_system_id(&mut self, input: &Chunk, ch: Option<u8>);

    fn finish_tag_name(&mut self, input: &Chunk, ch: Option<u8>);
    fn update_tag_name_hash(&mut self, input: &Chunk, ch: Option<u8>);
    fn mark_as_self_closing(&mut self, input: &Chunk, ch: Option<u8>);

    fn start_attr(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_attr_name(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_attr_value(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_attr(&mut self, input: &Chunk, ch: Option<u8>);

    fn set_closing_quote_to_double(&mut self, input: &Chunk, ch: Option<u8>);
    fn set_closing_quote_to_single(&mut self, input: &Chunk, ch: Option<u8>);

    fn notify_text_parsing_mode_change(
        &mut self,
        input: &Chunk,
        ch: Option<u8>,
        mode: TextParsingMode,
    );
}

pub trait StateMachineConditions {
    fn is_appropriate_end_tag(&self, ch: Option<u8>) -> bool;
    fn cdata_allowed(&self, ch: Option<u8>) -> bool;
}

pub trait StateMachine: StateMachineActions + StateMachineConditions {
    define_states!();

    fn set_state(&mut self, state: fn(&mut Self, &Chunk) -> StateResult);
    fn get_state(&self) -> fn(&mut Self, &Chunk) -> StateResult;
    fn get_input_cursor(&mut self) -> &mut Cursor;
    fn get_blocked_byte_count(&self, input: &Chunk) -> usize;
    fn adjust_for_next_input(&mut self);
    fn is_state_enter(&self) -> bool;
    fn set_is_state_enter(&mut self, val: bool);
    fn get_closing_quote(&self) -> u8;

    fn run_parsing_loop(&mut self, input: &Chunk) -> ParsingLoopResult {
        loop {
            let state = self.get_state();

            if let ParsingLoopDirective::Break(reason) = state(self, input)? {
                return Ok(reason);
            }
        }
    }

    fn break_on_end_of_input(&mut self, input: &Chunk) -> StateResult {
        let blocked_byte_count = self.get_blocked_byte_count(input);

        if !input.is_last() {
            self.adjust_for_next_input()
        }

        Ok(ParsingLoopDirective::Break(
            ParsingLoopTerminationReason::EndOfInput { blocked_byte_count },
        ))
    }

    #[cfg(feature = "testing_api")]
    fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>);

    #[cfg(feature = "testing_api")]
    fn set_text_parsing_mode_change_handler(
        &mut self,
        handler: Box<dyn TextParsingModeChangeHandler>,
    );

    #[inline]
    fn switch_state(&mut self, state: fn(&mut Self, &Chunk) -> StateResult) {
        self.set_state(state);
        self.set_is_state_enter(true);
    }

    #[inline]
    fn set_text_parsing_mode(&mut self, mode: TextParsingMode) {
        self.switch_state(match mode {
            TextParsingMode::Data => Self::data_state,
            TextParsingMode::PlainText => Self::plaintext_state,
            TextParsingMode::RCData => Self::rcdata_state,
            TextParsingMode::RawText => Self::rawtext_state,
            TextParsingMode::ScriptData => Self::script_data_state,
            TextParsingMode::CDataSection => Self::cdata_section_state,
        });
    }
}
