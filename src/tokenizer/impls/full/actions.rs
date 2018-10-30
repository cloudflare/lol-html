use super::*;
use base::Chunk;

macro_rules! get_token_part_range {
    ($self:tt) => {
        Range {
            start: $self.token_part_start,
            end: input!(@pos $self),
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

impl<H: LexUnitHandler> Tokenizer<H> {
    // Lex unit emission
    //--------------------------------------------------------------------
    #[inline]
    pub(super) fn emit_eof(&mut self, input: &Chunk, _ch: Option<u8>) {
        self.emit_lex_unit(input, Some(TokenView::Eof), None);
    }

    #[inline]
    pub(super) fn emit_chars(&mut self, input: &Chunk, _ch: Option<u8>) {
        if input!(@pos self) > self.lex_unit_start {
            // NOTE: unlike any other tokens, character tokens don't have
            // any lexical symbols that determine their bounds. Therefore,
            // representation of character token content is the raw slice.
            // Also, we always emit characters if we encounter some other bounded
            // lexical structure and, thus, we use exclusive range for the raw slice.
            self.emit_lex_unit_with_raw_exclusive(input, Some(TokenView::Character));
        }
    }

    #[inline]
    pub(super) fn emit_current_token(&mut self, input: &Chunk, _ch: Option<u8>) {
        let token = self.current_token.take();

        self.emit_lex_unit_with_raw_inclusive(input, token);
    }

    #[inline]
    pub(super) fn emit_tag(
        &mut self,
        input: &Chunk,
        _ch: Option<u8>,
    ) -> Result<Option<ParsingLoopDirective>, Error> {
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

        let lex_unit = self.emit_lex_unit_with_raw_inclusive(input, token);

        Ok(self.handle_tree_builder_feedback(feedback, lex_unit))
    }

    #[inline]
    pub(super) fn emit_current_token_and_eof(&mut self, input: &Chunk, ch: Option<u8>) {
        let token = self.current_token.take();

        self.emit_lex_unit_with_raw_exclusive(input, token);
        self.emit_eof(input, ch);
    }

    #[inline]
    pub(super) fn emit_raw_without_token(&mut self, input: &Chunk, _ch: Option<u8>) {
        self.emit_lex_unit_with_raw_inclusive(input, None);
    }

    #[inline]
    pub(super) fn emit_raw_without_token_and_eof(&mut self, input: &Chunk, ch: Option<u8>) {
        // NOTE: since we are at EOF we use exclusive range for token's raw.
        self.emit_lex_unit_with_raw_exclusive(input, None);
        self.emit_eof(input, ch);
    }

    // Token creation
    //--------------------------------------------------------------------
    #[inline]
    pub(super) fn create_start_tag(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.attr_buffer.borrow_mut().clear();

        self.current_token = Some(TokenView::StartTag {
            name: Range::default(),
            name_hash: Some(0),
            attributes: Rc::clone(&self.attr_buffer),
            self_closing: false,
        });
    }

    #[inline]
    pub(super) fn create_end_tag(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.current_token = Some(TokenView::EndTag {
            name: Range::default(),
            name_hash: Some(0),
        });
    }

    #[inline]
    pub(super) fn create_doctype(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.current_token = Some(TokenView::Doctype {
            name: None,
            public_id: None,
            system_id: None,
            force_quirks: false,
        });
    }

    #[inline]
    pub(super) fn create_comment(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.current_token = Some(TokenView::Comment(Range::default()));
    }

    // Token part
    //--------------------------------------------------------------------
    #[inline]
    pub(super) fn start_token_part(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.token_part_start = input!(@pos self);
    }

    // Comment parts
    //--------------------------------------------------------------------
    #[inline]
    pub(super) fn mark_comment_text_end(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::Comment(ref mut text)) = self.current_token {
            *text = get_token_part_range!(self);
        }
    }

    #[inline]
    pub(super) fn shift_comment_text_end_by(
        &mut self,
        _input: &Chunk,
        _ch: Option<u8>,
        offset: usize,
    ) {
        if let Some(TokenView::Comment(ref mut text)) = self.current_token {
            text.end += offset;
        }
    }

    // Doctype parts
    //--------------------------------------------------------------------
    #[inline]
    pub(super) fn set_force_quirks(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::Doctype {
            ref mut force_quirks,
            ..
        }) = self.current_token
        {
            *force_quirks = true;
        }
    }

    #[inline]
    pub(super) fn finish_doctype_name(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::Doctype { ref mut name, .. }) = self.current_token {
            *name = Some(get_token_part_range!(self));
        }
    }

    #[inline]
    pub(super) fn finish_doctype_public_id(&mut self, input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::Doctype {
            ref mut public_id, ..
        }) = self.current_token
        {
            *public_id = Some(get_token_part_range!(self));
        }
    }

    #[inline]
    pub(super) fn finish_doctype_system_id(&mut self, input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::Doctype {
            ref mut system_id, ..
        }) = self.current_token
        {
            *system_id = Some(get_token_part_range!(self));
        }
    }

    // Tag parts
    //--------------------------------------------------------------------
    #[inline]
    pub(super) fn finish_tag_name(&mut self, input: &Chunk, _ch: Option<u8>) {
        match self.current_token {
            Some(TokenView::StartTag { ref mut name, .. })
            | Some(TokenView::EndTag { ref mut name, .. }) => *name = get_token_part_range!(self),
            _ => unreachable!("Current token should be a start or an end tag at this point"),
        }
    }

    #[inline]
    pub(super) fn update_tag_name_hash(&mut self, _input: &Chunk, ch: Option<u8>) {
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
    pub(super) fn mark_as_self_closing(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(TokenView::StartTag {
            ref mut self_closing,
            ..
        }) = self.current_token
        {
            *self_closing = true;
        }
    }

    // Attributes
    //--------------------------------------------------------------------
    #[inline]
    pub(super) fn start_attr(&mut self, input: &Chunk, ch: Option<u8>) {
        // NOTE: create attribute only if we are parsing a start tag
        if let Some(TokenView::StartTag { .. }) = self.current_token {
            self.current_attr = Some(AttributeView::default());

            self.start_token_part(input, ch);
        }
    }

    #[inline]
    pub(super) fn finish_attr_name(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(AttributeView { ref mut name, .. }) = self.current_attr {
            *name = get_token_part_range!(self);
        }
    }

    #[inline]
    pub(super) fn finish_attr_value(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(AttributeView { ref mut value, .. }) = self.current_attr {
            *value = get_token_part_range!(self);
        }
    }

    #[inline]
    pub(super) fn finish_attr(&mut self, _input: &Chunk, _ch: Option<u8>) {
        if let Some(attr) = self.current_attr.take() {
            self.attr_buffer.borrow_mut().push(attr);
        }
    }

    // Quotes
    //--------------------------------------------------------------------
    #[inline]
    pub(super) fn set_closing_quote_to_double(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.closing_quote = b'"';
    }

    #[inline]
    pub(super) fn set_closing_quote_to_single(&mut self, _input: &Chunk, _ch: Option<u8>) {
        self.closing_quote = b'\'';
    }

    // Testing related
    //--------------------------------------------------------------------
    #[inline]
    pub(super) fn notify_text_parsing_mode_change(
        &mut self,
        _input: &Chunk,
        _ch: Option<u8>,
        mode: TextParsingMode,
    ) {
        notify_text_parsing_mode_change!(self, mode);
    }
}
