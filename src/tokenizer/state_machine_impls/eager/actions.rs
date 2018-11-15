use super::*;
use base::Chunk;
use tokenizer::{ParsingLoopDirective, StateMachineActions, StateResult};

macro_rules! noop {
    ($($fn_name:ident),*) => {
        $(
            #[inline]
            fn $fn_name(&mut self, _input: &Chunk, _ch: Option<u8>) { }
        )*
    };
}

impl<H> StateMachineActions<TagPreviewResponse> for EagerStateMachine<H>
where
    H: TagPreviewHandler,
{
    #[inline]
    fn create_start_tag(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.tag_name_start = self.input_cursor.pos();

        // NOTE: we are in the beginning of the start tag name.
        // The start of the tag is one byte behind ('<').
        self.tag_start = self.tag_name_start - 1;
        self.tag_name_hash = Some(0);
    }

    #[inline]
    fn create_end_tag(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.tag_name_start = self.input_cursor.pos();
        self.is_in_end_tag = true;

        // NOTE: we are in the beginning of the end tag name.
        // The start of the tag is two bytes behind ('</').
        self.tag_start = self.tag_name_start - 2;
        self.tag_name_hash = Some(0);
    }

    #[inline]
    fn update_tag_name_hash(&mut self, _input: &Chunk, ch: Option<u8>) {
        if let Some(ch) = ch {
            TagName::update_hash(&mut self.tag_name_hash, ch);
        }
    }

    #[inline]
    fn finish_tag_name(&mut self, input: &Chunk, _ch: Option<u8>) {
        let name_range = Range {
            start: self.tag_name_start,
            end: self.input_cursor.pos(),
        };

        let tag_name_info = TagNameInfo::new(input, name_range, self.tag_name_hash);

        let tag_preview = if self.is_in_end_tag {
            self.is_in_end_tag = false;
            TagPreview::EndTag(tag_name_info)
        } else {
            self.last_start_tag_name_hash = self.tag_name_hash;
            TagPreview::StartTag(tag_name_info)
        };

        self.tag_preview_handler.handle(&tag_preview);
    }

    #[inline]
    fn set_closing_quote_to_double(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.closing_quote = b'"';
    }

    #[inline]
    fn set_closing_quote_to_single(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.closing_quote = b'\'';
    }

    noop!(
        emit_eof,
        emit_chars,
        emit_current_token,
        emit_current_token_and_eof,
        emit_raw_without_token,
        emit_raw_without_token_and_eof,
        create_doctype,
        create_comment,
        start_token_part,
        mark_comment_text_end,
        set_force_quirks,
        finish_doctype_name,
        finish_doctype_public_id,
        finish_doctype_system_id,
        mark_as_self_closing,
        start_attr,
        finish_attr_name,
        finish_attr_value,
        finish_attr
    );

    #[inline]
    fn emit_tag(&mut self, _input: &Chunk, _ch: Option<u8>) -> StateResult<TagPreviewResponse> {
        Ok(ParsingLoopDirective::None)
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
