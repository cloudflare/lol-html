use super::*;
use crate::base::Chunk;
use crate::tokenizer::state_machine::{ParsingLoopDirective, StateMachineActions, StateResult};

impl<H: TagPreviewHandler> StateMachineActions for EagerStateMachine<H> {
    impl_common_sm_actions!();

    #[inline]
    fn create_start_tag(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        self.tag_name_start = self.input_cursor.pos();
        self.tag_name_hash = Some(0);
    }

    #[inline]
    fn create_end_tag(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        self.tag_name_start = self.input_cursor.pos();
        self.tag_name_hash = Some(0);
        self.is_in_end_tag = true;
    }

    #[inline]
    fn mark_tag_start(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        self.tag_start = Some(self.input_cursor.pos());
    }

    #[inline]
    fn unmark_tag_start(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        self.tag_start = None;
    }

    #[inline]
    fn update_tag_name_hash(&mut self, _input: &Chunk<'_>, ch: Option<u8>) {
        if let Some(ch) = ch {
            TagName::update_hash(&mut self.tag_name_hash, ch);
        }
    }

    #[inline]
    fn finish_tag_name(&mut self, input: &Chunk<'_>, _ch: Option<u8>) -> StateResult {
        let tag_preview = self.create_tag_preview(input);
        let next_output_type = self.tag_preview_handler.handle(&tag_preview);

        trace!(@output tag_preview);

        let tag_start = self
            .tag_start
            .take()
            .expect("Tag start should be set at this point");

        Ok(match next_output_type {
            NextOutputType::TagPreview => {
                let feedback = self.get_feedback_for_tag(&tag_preview)?;

                self.handle_tree_builder_feedback(feedback, tag_start)
            }
            NextOutputType::LexUnit => {
                // NOTE: we don't need to take feedback from tree builder simulator
                // here because tag will be re-parsed by the full state machine anyway.
                ParsingLoopDirective::Break(ParsingLoopTerminationReason::OutputTypeSwitch(
                    NextOutputType::LexUnit,
                    self.create_bookmark(tag_start),
                ))
            }
        })
    }

    #[inline]
    fn emit_tag(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) -> StateResult {
        confirm_tag!(self);

        Ok(
            if let Some(mode) = self.pending_text_parsing_mode_change.take() {
                self.switch_text_parsing_mode(mode);

                ParsingLoopDirective::Continue
            } else {
                // NOTE: exit from any non-initial text parsing mode always happens on tag emission
                // (except for CDATA, but there is a special action to take care of it).
                self.set_last_text_parsing_mode(TextParsingMode::Data);

                ParsingLoopDirective::None
            },
        )
    }

    noop_action!(
        emit_eof,
        emit_text,
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
    fn shift_comment_text_end_by(&mut self, _input: &Chunk<'_>, _ch: Option<u8>, _offset: usize) {
        trace!(@noop);
    }
}
