#[macro_use]
mod state_machine_dsl;

#[macro_use]
mod syntax;

#[macro_use]
mod tag_name;

mod full;
mod text_parsing_mode;
mod tree_builder_simulator;

use self::full::FullStateMachine;
use base::{Chunk, Cursor};
use errors::Error;

pub use self::full::{Attribute, LexUnit, LexUnitHandler, Token, TokenView};
pub use self::tag_name::TagName;
pub use self::text_parsing_mode::*;

pub enum ParsingLoopDirective {
    Break,
    Continue,
}

pub trait StateMachineActions {
    fn emit_eof(&mut self, input: &Chunk, ch: Option<u8>);
    fn emit_chars(&mut self, input: &Chunk, _ch: Option<u8>);
    fn emit_current_token(&mut self, input: &Chunk, ch: Option<u8>);

    fn emit_tag(
        &mut self,
        input: &Chunk,
        ch: Option<u8>,
    ) -> Result<Option<ParsingLoopDirective>, Error>;

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
    fn is_closing_quote(&self, ch: Option<u8>) -> bool;
}

pub trait StateMachine: StateMachineActions + StateMachineConditions {
    define_states!();

    #[inline]
    fn switch_state(
        &mut self,
        state: fn(&mut Self, &Chunk) -> Result<ParsingLoopDirective, Error>,
    ) {
        self.set_state(state);
        self.set_is_state_enter(true);
    }

    fn set_state(&mut self, state: fn(&mut Self, &Chunk) -> Result<ParsingLoopDirective, Error>);
    fn get_state(&self) -> fn(&mut Self, &Chunk) -> Result<ParsingLoopDirective, Error>;
    fn get_input_cursor(&mut self) -> &mut Cursor;
    fn get_blocked_byte_count(&self, input: &Chunk) -> usize;
    fn adjust_for_next_input(&mut self);
    fn is_state_enter(&self) -> bool;
    fn set_is_state_enter(&mut self, val: bool);

    fn set_text_parsing_mode(&mut self, mode: TextParsingMode);

    #[cfg(feature = "testing_api")]
    fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>);

    #[cfg(feature = "testing_api")]
    fn set_text_parsing_mode_change_handler(
        &mut self,
        handler: Box<dyn TextParsingModeChangeHandler>,
    );
}

pub struct Tokenizer<S: StateMachine>(S);

impl<S: StateMachine> Tokenizer<S> {
    pub fn tokenize(&mut self, input: &Chunk) -> Result<usize, Error> {
        loop {
            let state = self.0.get_state();
            let directive = state(&mut self.0, input)?;

            if let ParsingLoopDirective::Break = directive {
                break;
            }
        }

        let blocked_byte_count = self.0.get_blocked_byte_count(input);

        if !input.is_last() {
            self.0.adjust_for_next_input()
        }

        Ok(blocked_byte_count)
    }

    #[cfg(feature = "testing_api")]
    pub fn set_text_parsing_mode(&mut self, mode: TextParsingMode) {
        self.0.set_text_parsing_mode(mode);
    }

    #[cfg(feature = "testing_api")]
    pub fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        self.0.set_last_start_tag_name_hash(name_hash);
    }

    #[cfg(feature = "testing_api")]
    pub fn set_text_parsing_mode_change_handler(
        &mut self,
        handler: Box<dyn TextParsingModeChangeHandler>,
    ) {
        self.0.set_text_parsing_mode_change_handler(handler);
    }
}

pub type FullTokenizer<H> = Tokenizer<FullStateMachine<H>>;

impl<H: LexUnitHandler> FullTokenizer<H> {
    pub fn new(lex_unit_handler: H) -> Self {
        Tokenizer(FullStateMachine::new(lex_unit_handler))
    }
}
