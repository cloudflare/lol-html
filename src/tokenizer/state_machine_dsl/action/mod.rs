#[macro_use]
mod helpers;

#[macro_use]
mod state_transition;

#[macro_use]
mod emit_tag;

macro_rules! action {
    // Lex result emission
    //--------------------------------------------------------------------
    (| $self:tt, $input:ident, $ch:ident | > emit_eof) => {
        action_helper!(@emit_lex_unit |$self|> Some(TokenView::Eof), None, $input);
    };

    (| $self:tt, $input:ident, $ch:ident | > emit_chars) => {
        if input!(@pos $self) > $self.lex_unit_start {
            // NOTE: unlike any other tokens, character tokens don't have
            // any lexical symbols that determine their bounds. Therefore,
            // representation of character token content is the raw slice.
            // Also, we always emit characters if we encounter some other bounded
            // lexical structure and, thus, we use exclusive range for the raw slice.
            action_helper!(
                @emit_lex_unit_with_raw_exclusive |$self, $input|>
                Some(TokenView::Character)
            );
        }
    };

    (| $self:tt, $input:ident, $ch:ident | > emit_current_token) => {
        action_helper!(
            @emit_lex_unit_with_raw_inclusive |$self, $input|>
            $self.current_token.take()
        );
    };

    (| $self:tt, $input:ident, $ch:ident | > emit_tag) => {
        emit_tag!($self, $input)
    };

    (| $self:tt, $input:ident, $ch:ident | > emit_current_token_and_eof) => {
        match $self.current_token.take() {
            Some(token) => {
                // NOTE: we don't care about last_start_tag_name_hash here, since
                // we'll not produce any tokens besides EOF. Also, considering that
                // we are at EOF here we use exclusive range for token's raw.
                action_helper!(@emit_lex_unit_with_raw_exclusive |$self, $input|> Some(token));
            }
            None => unreachable!("Current token should exist at this point"),
        }

        action!(| $self, $input, $ch |> emit_eof);
    };

    (| $self:tt, $input:ident, $ch:ident | > emit_raw_without_token) => {
        action_helper!(@emit_lex_unit_with_raw_inclusive |$self, $input|> None);
    };

    (| $self:tt, $input:ident, $ch:ident | > emit_raw_without_token_and_eof) => {
        // NOTE: since we are at EOF we use exclusive range for token's raw.
        action_helper!(@emit_lex_unit_with_raw_exclusive |$self, $input|> None);
        action!(| $self, $input, $ch |> emit_eof);
    };

    // Slices
    //--------------------------------------------------------------------
    (| $self:tt, $input:ident, $ch:ident | > start_token_part) => {
        $self.token_part_start = input!(@pos $self);
    };

    // Token creation
    //--------------------------------------------------------------------
    (| $self:tt, $input:ident, $ch:ident | > create_start_tag) => {
        $self.attr_buffer.borrow_mut().clear();

        $self.current_token = Some(TokenView::StartTag {
            name: Range::default(),
            name_hash: Some(0),
            attributes: Rc::clone(&$self.attr_buffer),
            self_closing: false,
        });
    };

    (| $self:tt, $input:ident, $ch:ident | > create_end_tag) => {
        $self.current_token = Some(TokenView::EndTag {
            name: Range::default(),
            name_hash: Some(0),
        });
    };

    (| $self:tt, $input:ident, $ch:ident | > create_doctype) => {
        $self.current_token = Some(TokenView::Doctype {
            name: None,
            public_id: None,
            system_id: None,
            force_quirks: false,
        });
    };

    (| $self:tt, $input:ident, $ch:ident | > create_comment) => {
        $self.current_token = Some(TokenView::Comment(Range::default()));
    };

    // Comment parts
    //--------------------------------------------------------------------
    (| $self:tt, $input:ident, $ch:ident | > mark_comment_text_end) => {
        if let Some(TokenView::Comment(ref mut text)) = $self.current_token {
            action_helper!(@finish_token_part |$self, $input|> text);
        }
    };

    (| $self:tt, $input:ident, $ch:ident | > shift_comment_text_end_by $shift:expr) => {
        if let Some(TokenView::Comment(ref mut text)) = $self.current_token {
            text.end += $shift;
        }
    };

    // Doctype parts
    //--------------------------------------------------------------------
    (| $self:tt, $input:ident, $ch:ident | > set_force_quirks) => {
        if let Some(TokenView::Doctype {
            ref mut force_quirks,
            ..
        }) = $self.current_token
        {
            *force_quirks = true;
        }
    };

    (| $self:tt, $input:ident, $ch:ident | > finish_doctype_name) => {
        if let Some(TokenView::Doctype { ref mut name, .. }) = $self.current_token {
            action_helper!(@finish_opt_token_part |$self, $input|> name);
        }
    };

    (| $self:tt, $input:ident, $ch:ident | > finish_doctype_public_id) => {
        if let Some(TokenView::Doctype {
            ref mut public_id, ..
        }) = $self.current_token
        {
            action_helper!(@finish_opt_token_part |$self, $input|> public_id);
        }
    };

    (| $self:tt, $input:ident, $ch:ident | > finish_doctype_system_id) => {
        if let Some(TokenView::Doctype {
            ref mut system_id, ..
        }) = $self.current_token
        {
            action_helper!(@finish_opt_token_part |$self, $input|> system_id);
        }
    };

    // Tag parts
    //--------------------------------------------------------------------
    (| $self:tt, $input:ident, $ch:ident | > finish_tag_name) => {
        action_helper!(@update_tag_part |$self|> name,
            {
                action_helper!(@finish_token_part |$self, $input|> name);
            }
        );
    };

    (| $self:tt, $input:ident, $ch:ident | > update_tag_name_hash) => {
        if let Some(ch) = $ch {
            action_helper!(@update_tag_part |$self|> name_hash,
                {
                    TagName::update_hash(name_hash, ch);
                }
            );
        }
    };

    (| $self:tt, $input:ident, $ch:ident | > mark_as_self_closing) => {
        if let Some(TokenView::StartTag {
            ref mut self_closing,
            ..
        }) = $self.current_token
        {
            *self_closing = true;
        }
    };

    // Attributes
    //--------------------------------------------------------------------
    (| $self:tt, $input:ident, $ch:ident | > start_attr) => {
        // NOTE: create attribute only if we are parsing a start tag
        if let Some(TokenView::StartTag { .. }) = $self.current_token {
            $self.current_attr = Some(AttributeView::default());
            action!(|$self, $input, $ch|> start_token_part);
        }
    };

    (| $self:tt, $input:ident, $ch:ident | > finish_attr_name) => {
        action_helper!(@finish_attr_part |$self, $input|> name);
    };

    (| $self:tt, $input:ident, $ch:ident | > finish_attr_value) => {
        action_helper!(@finish_attr_part |$self, $input|> value);
    };

    (| $self:tt, $input:ident, $ch:ident | > finish_attr) => {
        if let Some(attr) = $self.current_attr.take() {
            $self.attr_buffer.borrow_mut().push(attr);
        }
    };

    // Quotes
    //--------------------------------------------------------------------
    (| $self:tt, $input:ident, $ch:ident | > set_closing_quote_to_double) => {
        $self.closing_quote = b'"';
    };

    (| $self:tt, $input:ident, $ch:ident | > set_closing_quote_to_single) => {
        $self.closing_quote = b'\'';
    };

    // Testing related
    //--------------------------------------------------------------------
    (| $self:tt, $input:ident, $ch:ident | > notify_text_parsing_mode_change $mode:expr) => {
        action_helper!(@notify_text_parsing_mode_change |$self|> $mode);
    };
}
