#[macro_use]
mod helpers;

#[macro_use]
mod state_transition;

macro_rules! action {
    ( | $self:ident |> emit_eof ) => {
        action_helper!(@emit_lex_result |$self|> ShallowToken::Eof, None);
        $self.finished = true;
    };

    ( | $self:ident |> emit_chars ) => {
        if $self.pos > $self.raw_start {
            // NOTE: unlike any other tokens, character tokens don't have
            // any lexical symbols that determine their bounds. Therefore,
            // representation of character token content is the raw slice.
            // Also, we always emit characters if we encounter some other bounded
            // lexical structure and, thus, we use exclusive range for the raw slice.
            action_helper!(@emit_lex_result_with_raw_exclusive |$self|> ShallowToken::Character);
        }
    };

    ( | $self:ident |> emit_comment ) => {
        let mut text = SliceRange::default();

        action_helper!(@set_token_part_range |$self|> text);
        action_helper!(@emit_lex_result_with_raw_inclusive |$self|> ShallowToken::Comment(text));
    };

    ( | $self:ident |> start_raw ) => {
        $self.raw_start = $self.pos;
    };

    ( | $self:ident |> start_token_part ) => {
        $self.token_part_start = $self.pos - $self.raw_start;
    };

    ( | $self:ident |> create_start_tag ) => {
        $self.attr_buffer.borrow_mut().clear();

        $self.current_token = Some(ShallowToken::StartTag {
            name: SliceRange::default(),
            attributes: Rc::clone(&$self.attr_buffer),
            self_closing: false,
        });
    };

    ( | $self:ident |> finish_tag_name ) => {
        match $self.current_token {
            Some(ShallowToken::StartTag { ref mut name, .. }) => {
                action_helper!(@set_token_part_range |$self|> name);
            }
            _ => unreachable!("Current token should always be a start tag at this point")
        }
    };

    ( | $self: ident |> emit_current_token ) => {
        match $self.current_token.take() {
            Some(token) => {
                $self.current_token = None;
                action_helper!(@emit_lex_result_with_raw_inclusive |$self|> token);
            }
            None => unreachable!("Current token should be already created at this point")
        }
    };
}
