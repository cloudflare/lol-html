use super::*;
use base::Chunk;
use tokenizer::{OutputResponseResult, StateMachineActions};

macro_rules! get_token_part_range {
    ($self:tt) => {
        Range {
            start: $self.token_part_start,
            end: $self.input_cursor.pos(),
        }
    };
}

macro_rules! notify_text_parsing_mode_change {
    ($self:tt, $mode:ident) => {
        #[cfg(feature = "testing_api")]
        {
            if let Some(ref mut text_parsing_mode_change_handler) =
                $self.text_parsing_mode_change_handler
            {
                text_parsing_mode_change_handler.handle(TextParsingModeSnapshot {
                    mode: $mode,
                    last_start_tag_name_hash: $self.last_start_tag_name_hash,
                });
            }
        }
    };
}

impl<LH, TH> StateMachineActions for FullStateMachine<LH, TH>
where
    LH: LexUnitHandler,
    TH: TagLexUnitHandler,
{
    #[inline]
    fn emit_eof(&mut self, input: &Chunk, _ch: Option<u8>) {
        self.emit_lex_unit(input, Some(TokenView::Eof), None);
    }

    #[inline]
    fn emit_chars(&mut self, input: &Chunk, _ch: Option<u8>) {
        if self.input_cursor.pos() > self.lex_unit_start {
            // NOTE: unlike any other tokens, character tokens don't have
            // any lexical symbols that determine their bounds. Therefore,
            // representation of character token content is the raw slice.
            // Also, we always emit characters if we encounter some other bounded
            // lexical structure and, thus, we use exclusive range for the raw slice.
            self.emit_lex_unit_with_raw_exclusive(input, Some(TokenView::Character));
        }
    }

    #[inline]
    fn emit_current_token(&mut self, input: &Chunk, _ch: Option<u8>) {
        let token = self.current_token.take();

        self.emit_lex_unit_with_raw_inclusive(input, token);
    }

    #[inline]
    fn emit_tag(&mut self, input: &Chunk, _ch: Option<u8>) -> OutputResponseResult {
        let token = self.current_token.take();

        let feedback = match token {
            Some(TokenView::StartTag { name_hash, .. }) => {
                self.last_start_tag_name_hash = name_hash;
                self.tree_builder_simulator
                    .get_feedback_for_start_tag_name(name_hash)?
            }
            Some(TokenView::EndTag { name_hash, .. }) => self
                .tree_builder_simulator
                .get_feedback_for_end_tag_name(name_hash),
            _ => unreachable!("Token should be a start or an end tag at this point"),
        };

        let (lex_unit, _response) = self.emit_tag_lex_unit(input, token);

        Ok(self.handle_tree_builder_feedback(feedback, &lex_unit))
    }

    #[inline]
    fn emit_current_token_and_eof(&mut self, input: &Chunk, ch: Option<u8>) {
        let token = self.current_token.take();

        self.emit_lex_unit_with_raw_exclusive(input, token);
        self.emit_eof(input, ch);
    }

    #[inline]
    fn emit_raw_without_token(&mut self, input: &Chunk, _ch: Option<u8>) {
        self.emit_lex_unit_with_raw_inclusive(input, None);
    }

    #[inline]
    fn emit_raw_without_token_and_eof(&mut self, input: &Chunk, ch: Option<u8>) {
        // NOTE: since we are at EOF we use exclusive range for token's raw.
        self.emit_lex_unit_with_raw_exclusive(input, None);
        self.emit_eof(input, ch);
    }

    #[inline]
    fn create_start_tag(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.attr_buffer.borrow_mut().clear();

        self.current_token = Some(TokenView::StartTag {
            name: Range::default(),
            name_hash: Some(0),
            attributes: Rc::clone(&self.attr_buffer),
            self_closing: false,
        });
    }

    #[inline]
    fn create_end_tag(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.current_token = Some(TokenView::EndTag {
            name: Range::default(),
            name_hash: Some(0),
        });
    }

    #[inline]
    fn create_doctype(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.current_token = Some(TokenView::Doctype {
            name: None,
            public_id: None,
            system_id: None,
            force_quirks: false,
        });
    }

    #[inline]
    fn create_comment(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.current_token = Some(TokenView::Comment(Range::default()));
    }

    #[inline]
    fn start_token_part(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.token_part_start = self.input_cursor.pos();
    }

    #[inline]
    fn mark_comment_text_end(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::Comment(ref mut text)) = self.current_token {
            *text = get_token_part_range!(self);
        }
    }

    #[inline]
    fn shift_comment_text_end_by(&mut self, _input: &Chunk, _ch: Option<u8>, offset: usize) {
        if let Some(TokenView::Comment(ref mut text)) = self.current_token {
            text.end += offset;
        }
    }

    #[inline]
    fn set_force_quirks(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::Doctype {
            ref mut force_quirks,
            ..
        }) = self.current_token
        {
            *force_quirks = true;
        }
    }

    #[inline]
    fn finish_doctype_name(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::Doctype { ref mut name, .. }) = self.current_token {
            *name = Some(get_token_part_range!(self));
        }
    }

    #[inline]
    fn finish_doctype_public_id(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::Doctype {
            ref mut public_id, ..
        }) = self.current_token
        {
            *public_id = Some(get_token_part_range!(self));
        }
    }

    #[inline]
    fn finish_doctype_system_id(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::Doctype {
            ref mut system_id, ..
        }) = self.current_token
        {
            *system_id = Some(get_token_part_range!(self));
        }
    }

    #[inline]
    fn finish_tag_name(&mut self, _input: &Chunk, _ch: Option<u8>) {
        match self.current_token {
            Some(TokenView::StartTag { ref mut name, .. })
            | Some(TokenView::EndTag { ref mut name, .. }) => *name = get_token_part_range!(self),
            _ => unreachable!("Current token should be a start or an end tag at this point"),
        }
    }

    #[inline]
    fn update_tag_name_hash(&mut self, _input: &Chunk, ch: Option<u8>) {
        if let Some(ch) = ch {
            match self.current_token {
                Some(TokenView::StartTag {
                    ref mut name_hash, ..
                })
                | Some(TokenView::EndTag {
                    ref mut name_hash, ..
                }) => TagName::update_hash(name_hash, ch),
                _ => unreachable!("Current token should be a start or an end tag at this point"),
            }
        }
    }

    #[inline]
    fn mark_as_self_closing(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::StartTag {
            ref mut self_closing,
            ..
        }) = self.current_token
        {
            *self_closing = true;
        }
    }

    #[inline]
    fn start_attr(&mut self, input: &Chunk, ch: Option<u8>) {
        // NOTE: create attribute only if we are parsing a start tag
        if let Some(TokenView::StartTag { .. }) = self.current_token {
            self.current_attr = Some(AttributeView::default());

            self.start_token_part(input, ch);
        }
    }

    #[inline]
    fn finish_attr_name(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(AttributeView { ref mut name, .. }) = self.current_attr {
            *name = get_token_part_range!(self);
        }
    }

    #[inline]
    fn finish_attr_value(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(AttributeView { ref mut value, .. }) = self.current_attr {
            *value = get_token_part_range!(self);
        }
    }

    #[inline]
    fn finish_attr(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(attr) = self.current_attr.take() {
            self.attr_buffer.borrow_mut().push(attr);
        }
    }

    #[inline]
    fn set_closing_quote_to_double(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.closing_quote = b'"';
    }

    #[inline]
    fn set_closing_quote_to_single(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.closing_quote = b'\'';
    }

    #[inline]
    // NOTE: prevent compiler from complaining about unused `mode` in release builds.
    #[allow(unused_variables)]
    fn notify_text_parsing_mode_change(
        &mut self,
        _input: &Chunk,
        _ch: Option<u8>,
        mode: TextParsingMode,
    ) {
        notify_text_parsing_mode_change!(self, mode);
    }
}
