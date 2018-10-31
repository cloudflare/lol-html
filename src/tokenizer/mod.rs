#[macro_use]
mod state_machine_dsl;

#[macro_use]
mod syntax;

#[macro_use]
mod tag_name;

mod full;
mod tree_builder_simulator;

pub use self::full::*;
use self::tree_builder_simulator::TextParsingMode;
use base::Chunk;
use errors::Error;

pub(self) trait StateMachineActions {
    // Lex unit emission
    //--------------------------------------------------------------------
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

    // Token creation
    //--------------------------------------------------------------------
    fn create_start_tag(&mut self, input: &Chunk, ch: Option<u8>);
    fn create_end_tag(&mut self, input: &Chunk, ch: Option<u8>);
    fn create_doctype(&mut self, input: &Chunk, ch: Option<u8>);
    fn create_comment(&mut self, input: &Chunk, ch: Option<u8>);

    // Token part
    //--------------------------------------------------------------------
    fn start_token_part(&mut self, input: &Chunk, ch: Option<u8>);

    // Comment parts
    //--------------------------------------------------------------------
    fn mark_comment_text_end(&mut self, input: &Chunk, ch: Option<u8>);
    fn shift_comment_text_end_by(&mut self, input: &Chunk, ch: Option<u8>, offset: usize);

    // Doctype parts
    //--------------------------------------------------------------------
    fn set_force_quirks(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_doctype_name(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_doctype_public_id(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_doctype_system_id(&mut self, input: &Chunk, ch: Option<u8>);

    // Tag parts
    //--------------------------------------------------------------------
    fn finish_tag_name(&mut self, input: &Chunk, ch: Option<u8>);
    fn update_tag_name_hash(&mut self, input: &Chunk, ch: Option<u8>);
    fn mark_as_self_closing(&mut self, input: &Chunk, ch: Option<u8>);

    // Attributes
    //--------------------------------------------------------------------
    fn start_attr(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_attr_name(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_attr_value(&mut self, input: &Chunk, ch: Option<u8>);
    fn finish_attr(&mut self, input: &Chunk, ch: Option<u8>);

    // Quotes
    //--------------------------------------------------------------------
    fn set_closing_quote_to_double(&mut self, input: &Chunk, ch: Option<u8>);
    fn set_closing_quote_to_single(&mut self, input: &Chunk, ch: Option<u8>);

    // Testing related
    //--------------------------------------------------------------------
    fn notify_text_parsing_mode_change(
        &mut self,
        input: &Chunk,
        ch: Option<u8>,
        mode: TextParsingMode,
    );
}

pub(self) trait StateMachineConditions {
    fn is_appropriate_end_tag(&self) -> bool;
    fn cdata_allowed(&self) -> bool;
}

pub(self) trait StateMachine: StateMachineActions + StateMachineConditions {
    //define_state_machine!();
}
