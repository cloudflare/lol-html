use super::*;
use crate::base::Chunk;
use crate::parser::state_machine::{ParsingLoopDirective, StateMachineActions, StateResult};

impl<S: TagHintSink> StateMachineActions for TagScanner<S> {
    impl_common_sm_actions!();

    #[inline]
    fn create_start_tag(&mut self, input: &mut Chunk) {
        self.tag_name_start = input.pos();
        self.tag_name_hash = LocalNameHash::new();
    }

    #[inline]
    fn create_end_tag(&mut self, input: &mut Chunk) {
        self.tag_name_start = input.pos();
        self.tag_name_hash = LocalNameHash::new();
        self.is_in_end_tag = true;
    }

    #[inline]
    fn mark_tag_start(&mut self, input: &mut Chunk) {
        self.tag_start = Some(input.pos());
    }

    #[inline]
    fn unmark_tag_start(&mut self, _input: &mut Chunk) {
        self.tag_start = None;
    }

    #[inline]
    fn update_tag_name_hash(&mut self, input: &mut Chunk) {
        if let Some(ch) = input.get(input.pos()) {
            self.tag_name_hash.update(ch);
        }
    }

    #[inline]
    fn finish_tag_name(&mut self, input: &mut Chunk) -> StateResult {
        let tag_start = self
            .tag_start
            .take()
            .expect("Tag start should be set at this point");

        let unhandled_feedback = self
            .try_apply_tree_builder_feedback()
            .map_err(RewritingError::ParsingAmbiguity)?;

        Ok(match unhandled_feedback {
            Some(unhandled_feedback) => self.change_parser_directive(
                tag_start,
                ParserDirective::Lex,
                FeedbackDirective::ApplyUnhandledFeedback(unhandled_feedback),
            ),
            None => match self.emit_tag_hint(input)? {
                ParserDirective::WherePossibleScanForTagsOnly => ParsingLoopDirective::None,
                ParserDirective::Lex => {
                    let feedback_directive = match self.pending_text_type_change.take() {
                        Some(text_type) => FeedbackDirective::ApplyUnhandledFeedback(
                            TreeBuilderFeedback::SwitchTextType(text_type),
                        ),
                        None => FeedbackDirective::Skip,
                    };

                    self.change_parser_directive(
                        tag_start,
                        ParserDirective::Lex,
                        feedback_directive,
                    )
                }
            },
        })
    }

    #[inline]
    fn emit_tag(&mut self, _input: &mut Chunk) -> StateResult {
        Ok(
            if let Some(text_type) = self.pending_text_type_change.take() {
                self.switch_text_type(text_type);

                ParsingLoopDirective::Continue
            } else {
                // NOTE: exit from any non-initial text parsing mode always happens on tag emission
                // (except for CDATA, but there is a special action to take care of it).
                self.set_last_text_type(TextType::Data);

                ParsingLoopDirective::None
            },
        )
    }

    noop_action_with_result!(
        emit_eof,
        emit_text,
        emit_current_token,
        emit_current_token_and_eof,
        emit_raw_without_token,
        emit_raw_without_token_and_eof
    );

    noop_action!(
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
    fn shift_comment_text_end_by(&mut self, _input: &mut Chunk, _offset: usize) {
        trace!(@noop);
    }
}
