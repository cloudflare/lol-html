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
