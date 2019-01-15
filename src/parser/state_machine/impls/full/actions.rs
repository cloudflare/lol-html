use super::*;
use crate::base::Chunk;
use crate::parser::state_machine::StateMachineActions;

macro_rules! get_token_part_range {
    ($self:tt) => {
        Range {
            start: $self.token_part_start,
            end: $self.input_cursor.pos(),
        }
    };
}

impl<S: LexemeSink> StateMachineActions for FullStateMachine<S> {
    impl_common_sm_actions!();

    #[inline]
    fn emit_eof(&mut self, input: &Chunk<'_>, _ch: Option<u8>) {
        let lexeme = self.create_lexeme_with_raw_exclusive(input, Some(TokenOutline::Eof));

        self.emit_lexeme(&lexeme);
    }

    #[inline]
    fn emit_text(&mut self, input: &Chunk<'_>, _ch: Option<u8>) {
        if self.input_cursor.pos() > self.lexeme_start {
            // NOTE: unlike any other tokens, text tokens (except EOF) don't have
            // any lexical symbols that determine their bounds. Therefore,
            // representation of text token content is the raw slice.
            // Also, we always emit text if we encounter some other bounded
            // lexical structure and, thus, we use exclusive range for the raw slice.
            let lexeme = self.create_lexeme_with_raw_exclusive(
                input,
                Some(TokenOutline::Text(self.last_text_type)),
            );

            self.emit_lexeme(&lexeme);
        }
    }

    #[inline]
    fn emit_current_token(&mut self, input: &Chunk<'_>, _ch: Option<u8>) {
        let token = self.current_token.take();
        let lexeme = self.create_lexeme_with_raw_inclusive(input, token);

        self.emit_lexeme(&lexeme);
    }

    #[inline]
    fn emit_tag(&mut self, input: &Chunk<'_>, _ch: Option<u8>) -> StateResult {
        let token = self.current_token.take();

        if let Some(TokenOutline::StartTag { name_hash, .. }) = token {
            self.last_start_tag_name_hash = name_hash;
        }

        let feedback = self.get_feedback_for_tag(&token)?;
        let lexeme = self.create_lexeme_with_raw_inclusive(input, token);
        let next_output_type = self.emit_tag_lexeme(&lexeme);

        // NOTE: exit from any non-initial text parsing mode always happens on tag emission
        // (except for CDATA, but there is a special action to take care of it).
        self.set_last_text_type(TextType::Data);

        let loop_directive_from_feedback = self.handle_tree_builder_feedback(feedback, &lexeme);

        Ok(match next_output_type {
            NextOutputType::Lexeme => loop_directive_from_feedback,
            NextOutputType::TagHint => {
                ParsingLoopDirective::Break(ParsingLoopTerminationReason::OutputTypeSwitch(
                    NextOutputType::TagHint,
                    self.create_bookmark(self.lexeme_start),
                ))
            }
        })
    }

    #[inline]
    fn emit_current_token_and_eof(&mut self, input: &Chunk<'_>, ch: Option<u8>) {
        let token = self.current_token.take();
        let lexeme = self.create_lexeme_with_raw_exclusive(input, token);

        self.emit_lexeme(&lexeme);
        self.emit_eof(input, ch);
    }

    #[inline]
    fn emit_raw_without_token(&mut self, input: &Chunk<'_>, _ch: Option<u8>) {
        let lexeme = self.create_lexeme_with_raw_inclusive(input, None);

        self.emit_lexeme(&lexeme);
    }

    #[inline]
    fn emit_raw_without_token_and_eof(&mut self, input: &Chunk<'_>, ch: Option<u8>) {
        // NOTE: since we are at EOF we use exclusive range for token's raw.
        let lexeme = self.create_lexeme_with_raw_exclusive(input, None);

        self.emit_lexeme(&lexeme);
        self.emit_eof(input, ch);
    }

    #[inline]
    fn create_start_tag(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        self.attr_buffer.borrow_mut().clear();

        self.current_token = Some(TokenOutline::StartTag {
            name: Range::default(),
            name_hash: Some(0),
            attributes: Rc::clone(&self.attr_buffer),
            self_closing: false,
        });
    }

    #[inline]
    fn create_end_tag(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        self.current_token = Some(TokenOutline::EndTag {
            name: Range::default(),
            name_hash: Some(0),
        });
    }

    #[inline]
    fn create_doctype(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        self.current_token = Some(TokenOutline::Doctype {
            name: None,
            public_id: None,
            system_id: None,
            force_quirks: false,
        });
    }

    #[inline]
    fn create_comment(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        self.current_token = Some(TokenOutline::Comment(Range::default()));
    }

    #[inline]
    fn start_token_part(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        self.token_part_start = self.input_cursor.pos();
    }

    #[inline]
    fn mark_comment_text_end(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        if let Some(TokenOutline::Comment(ref mut text)) = self.current_token {
            *text = get_token_part_range!(self);
        }
    }

    #[inline]
    fn shift_comment_text_end_by(&mut self, _input: &Chunk<'_>, _ch: Option<u8>, offset: usize) {
        if let Some(TokenOutline::Comment(ref mut text)) = self.current_token {
            text.end += offset;
        }
    }

    #[inline]
    fn set_force_quirks(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        if let Some(TokenOutline::Doctype {
            ref mut force_quirks,
            ..
        }) = self.current_token
        {
            *force_quirks = true;
        }
    }

    #[inline]
    fn finish_doctype_name(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        if let Some(TokenOutline::Doctype { ref mut name, .. }) = self.current_token {
            *name = Some(get_token_part_range!(self));
        }
    }

    #[inline]
    fn finish_doctype_public_id(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        if let Some(TokenOutline::Doctype {
            ref mut public_id, ..
        }) = self.current_token
        {
            *public_id = Some(get_token_part_range!(self));
        }
    }

    #[inline]
    fn finish_doctype_system_id(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        if let Some(TokenOutline::Doctype {
            ref mut system_id, ..
        }) = self.current_token
        {
            *system_id = Some(get_token_part_range!(self));
        }
    }

    #[inline]
    fn finish_tag_name(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) -> StateResult {
        match self.current_token {
            Some(TokenOutline::StartTag { ref mut name, .. })
            | Some(TokenOutline::EndTag { ref mut name, .. }) => {
                *name = get_token_part_range!(self)
            }
            _ => unreachable!("Current token should be a start or an end tag at this point"),
        }

        Ok(ParsingLoopDirective::None)
    }

    #[inline]
    fn update_tag_name_hash(&mut self, _input: &Chunk<'_>, ch: Option<u8>) {
        if let Some(ch) = ch {
            match self.current_token {
                Some(TokenOutline::StartTag {
                    ref mut name_hash, ..
                })
                | Some(TokenOutline::EndTag {
                    ref mut name_hash, ..
                }) => TagName::update_hash(name_hash, ch),
                _ => unreachable!("Current token should be a start or an end tag at this point"),
            }
        }
    }

    #[inline]
    fn mark_as_self_closing(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        if let Some(TokenOutline::StartTag {
            ref mut self_closing,
            ..
        }) = self.current_token
        {
            *self_closing = true;
        }
    }

    #[inline]
    fn start_attr(&mut self, input: &Chunk<'_>, ch: Option<u8>) {
        // NOTE: create attribute only if we are parsing a start tag
        if let Some(TokenOutline::StartTag { .. }) = self.current_token {
            self.current_attr = Some(AttributeOultine::default());

            self.start_token_part(input, ch);
        }
    }

    #[inline]
    fn finish_attr_name(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        if let Some(AttributeOultine { ref mut name, .. }) = self.current_attr {
            *name = get_token_part_range!(self);
        }
    }

    #[inline]
    fn finish_attr_value(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        if let Some(AttributeOultine { ref mut value, .. }) = self.current_attr {
            *value = get_token_part_range!(self);
        }
    }

    #[inline]
    fn finish_attr(&mut self, _input: &Chunk<'_>, _ch: Option<u8>) {
        if let Some(attr) = self.current_attr.take() {
            self.attr_buffer.borrow_mut().push(attr);
        }
    }

    noop_action!(mark_tag_start, unmark_tag_start);
}
