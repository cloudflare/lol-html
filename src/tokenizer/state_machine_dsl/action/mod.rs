#[macro_use]
mod helpers;

#[macro_use]
mod state_transition;

macro_rules! action {
    ( | $me:ident |> emit_eof ) => {
        action_helper!(@emit_lex_result |$me|> ShallowToken::Eof, None);
        $me.finished = true;
    };

    ( | $me:ident |> emit_chars ) => {
        if $me.pos > $me.raw_start {
            // NOTE: unlike any other tokens, character tokens don't have
            // any lexical symbols that determine their bounds. Therefore,
            // representation of character token content is the raw slice.
            // Also, we always emit characters if we encounter some other bounded
            // lexical structure and, thus, we use exclusive range for the raw slice.
            action_helper!(@emit_lex_result_with_raw_exclusive |$me|> ShallowToken::Character);
        }
    };

    ( | $me:ident |> emit_comment ) => {
        let mut text = SliceRange::default();

        action_helper!(@set_token_part_range |$me|> text);
        action_helper!(@emit_lex_result_with_raw_inclusive |$me|> ShallowToken::Comment(text));
    };

    ( | $me:ident |> start_raw ) => {
        $me.raw_start = $me.pos;
    };

    ( | $me:ident |> start_token_part ) => {
        $me.token_part_start = $me.pos - $me.raw_start;
    };

    ( | $me:ident |> create_start_tag ) => {
        $me.attr_buffer.borrow_mut().clear();

        $me.current_token = Some(ShallowToken::StartTag {
            name: SliceRange::default(),
            attributes: Rc::clone(&$me.attr_buffer),
            self_closing: false,
        });
    };

    ( | $me:ident |> finish_tag_name ) => {
        match $me.current_token {
            Some(ShallowToken::StartTag { ref mut name, .. }) => {
                action_helper!(@set_token_part_range |$me|> name);
            }
            _ => unreachable!("Current token should always be a start tag at this point")
        }
    };

    ( | $me: ident |> emit_current_token ) => {
        match $me.current_token.take() {
            Some(token) => {
                $me.current_token = None;
                action_helper!(@emit_lex_result_with_raw_inclusive |$me|> token);
            }
            None => unreachable!("Current token should be already created at this point")
        }
    };
}
