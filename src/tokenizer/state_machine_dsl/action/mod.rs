#[macro_use]
mod helpers;

#[macro_use]
mod state_transition;

macro_rules! action {
    ( | $me:ident |> emit_eof ) => {
        action_helper!(@emit_lex_result |$me|> ShallowToken::Eof, None);
        $me.finished = true;
    };

    ( | $me:ident |> emit_chars ) => ( action_helper!(@emit_textual_token |$me|> Character); );

    ( | $me:ident |> emit_comment ) => ( action_helper!(@emit_textual_token |$me|> Comment); );

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
                action_helper!(@emit_lex_result |$me|> token);
            }
            None => unreachable!("Current token should be already created at this point")
        }
    };
}
