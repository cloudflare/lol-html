use super::*;
use base::Chunk;
use tokenizer::state_machine::{ParsingLoopDirective, StateMachineActions, StateResult};

impl<H: TagPreviewHandler> StateMachineActions for EagerStateMachine<H> {
    impl_common_sm_actions!();

    #[inline]
    fn create_start_tag(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.tag_name_start = self.input_cursor.pos();
        self.tag_name_hash = Some(0);
    }

    #[inline]
    fn create_end_tag(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.tag_name_start = self.input_cursor.pos();
        self.tag_name_hash = Some(0);
        self.is_in_end_tag = true;
    }

    #[inline]
    fn mark_tag_start(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.tag_start = Some(self.input_cursor.pos());
    }

    #[inline]
    fn update_tag_name_hash(&mut self, _input: &Chunk, ch: Option<u8>) {
        if let Some(ch) = ch {
            TagName::update_hash(&mut self.tag_name_hash, ch);
        }
    }

    #[inline]
    fn finish_tag_name(&mut self, input: &Chunk, _ch: Option<u8>) -> StateResult {
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

        let tag_start = self
            .tag_start
            .expect("Tag start should be set at this point");

        self.tag_start = None;

        let next_output_type = self.tag_preview_handler.handle(&tag_preview);

        Ok(match next_output_type {
            NextOutputType::TagPreview => ParsingLoopDirective::None,
            NextOutputType::LexUnit => {
                ParsingLoopDirective::Break(ParsingLoopTerminationReason::OutputTypeSwitch {
                    next_type: NextOutputType::LexUnit,
                    sm_bookmark: self.create_bookmark(tag_start),
                })
            }
        })
    }

    #[inline]
    fn emit_tag(&mut self, _input: &Chunk, _ch: Option<u8>) -> StateResult {
        Ok(ParsingLoopDirective::None)
    }

    noop_action!(
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

    // NOTE: Noop
    #[inline]
    fn shift_comment_text_end_by(&mut self, _input: &Chunk, _ch: Option<u8>, _offset: usize) {}
}
