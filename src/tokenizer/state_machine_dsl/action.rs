macro_rules! action {
    ( | $me:ident |> emit_eof ) => {
        let res = LexResult {
            token_descr: TokenDescriptor::Eof,
            raw: None,
        };

        ($me.token_handler)(res);
        $me.finished = true;
    };

    ( | $me:ident |> emit_chars ) => {
        if $me.pos > $me.raw_start {
            let res = LexResult {
                token_descr: TokenDescriptor::Character,
                raw: Some(&$me.buffer[$me.raw_start..$me.pos]),
            };

            ($me.token_handler)(res);
        }
    };

    ( | $me:ident |> start_raw ) => {
        $me.raw_start = $me.pos;
    };

    ( | $me:ident |> create_start_tag ) => {
        // TODO
    };

    // State transition actions
    //--------------------------------------------------------------------
    ( @state_transition | $me:ident |> reconsume in $state:ident ) => {
        $me.pos -= 1;
        state_transition!(| $me |> --> $state);
    };

    ( @state_transition | $me:ident |> --> $state:ident ) => {
        $me.state = Tokenizer::$state;
        $me.state_enter = true;
        return;
    };
}
