macro_rules! action {
    ( | $me:ident |> emit_eof ) => {
        let res = LexResult {
            shallow_token: ShallowToken::Eof,
            raw: None,
        };

        ($me.token_handler)(res);

        $me.finished = true;
    };

    ( | $me:ident |> emit_chars ) => ( action!(@helper |$me|> emit_textual_token Character); );

    ( | $me:ident |> emit_comment ) => ( action!(@helper |$me|> emit_textual_token Comment); );

    ( | $me:ident |> start_raw ) => {
        $me.raw_start = $me.pos;
    };

    ( | $me:ident |> start_slice ) => {
        $me.slice_start = $me.pos;
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
                (*name).start = $me.slice_start - $me.raw_start;
                (*name).end = $me.pos;
            }
            _ => unreachable!("Current token should always be a start tag at this point")
        }
    };

    ( | $me: ident |> emit_current_token ) => {
        match $me.current_token.take() {
            Some(token) => {
                $me.current_token = None;

                let res = LexResult {
                    shallow_token: token,
                    raw: Some(&$me.buffer[$me.raw_start..=$me.pos]),
                };

                ($me.token_handler)(res);
            }
            None => unreachable!("Current token should be already created at this point")
        }
    };


    // State transition actions
    //--------------------------------------------------------------------
    ( @state_transition | $me:ident |> reconsume in $state:ident ) => {
        $me.pos -= 1;
        action!(@state_transition | $me |> --> $state);
    };

    ( @state_transition | $me:ident |> --> $state:ident ) => {
        $me.state = Tokenizer::$state;
        $me.state_enter = true;
        return;
    };


    // Internal helpers
    //--------------------------------------------------------------------
    ( @helper | $me:ident |> emit_textual_token $ty:ident ) => {
        if $me.pos > $me.raw_start {
            let res = LexResult {
                shallow_token: ShallowToken::$ty,
                raw: Some(&$me.buffer[$me.raw_start..$me.pos]),
            };

            ($me.token_handler)(res);
        }
    };
}
