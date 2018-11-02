use super::*;
use base::Chunk;
use tokenizer::StateMachineActions;

macro_rules! noop {
    ($($fn_name:ident),*) => {
        $(
            #[inline]
            fn $fn_name(&mut self, _input: &Chunk, _ch: Option<u8>) { }
        )*
    };
}

impl<H: TagPreviewHandler> StateMachineActions for EagerStateMachine<H> {
    noop!(
        emit_eof,
        emit_chars,
        emit_current_token,
        emit_current_token_and_eof,
        emit_raw_without_token,
        emit_raw_without_token_and_eof,
        create_start_tag,
        create_end_tag,
        create_doctype,
        create_comment,
        start_token_part,
        mark_comment_text_end,
        set_force_quirks,
        finish_doctype_name,
        finish_doctype_public_id,
        finish_doctype_system_id,
        finish_tag_name,
        update_tag_name_hash,
        mark_as_self_closing,
        start_attr,
        finish_attr_name,
        finish_attr_value,
        finish_attr,
        set_closing_quote_to_double,
        set_closing_quote_to_single
    );

    #[inline]
    fn emit_tag(
        &mut self,
        input: &Chunk,
        ch: Option<u8>,
    ) -> Result<Option<ParsingLoopDirective>, Error> {
        Ok(None)
    }

    #[inline]
    fn notify_text_parsing_mode_change(
        &mut self,
        _input: &Chunk,
        _ch: Option<u8>,
        _mode: TextParsingMode,
    ) {
        // Noop
    }

    #[inline]
    fn shift_comment_text_end_by(&mut self, _input: &Chunk, _ch: Option<u8>, _offset: usize) {
        // Noop
    }
}
