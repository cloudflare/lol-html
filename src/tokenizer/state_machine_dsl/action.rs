macro_rules! action {
    ( | $me:ident |> emit_eof ) => {
        ($me.token_handler)(Token::Eof, None);
        $me.finished = true;
    };

    ( | $me:ident |> emit_chars ) => {
        if $me.pos > $me.raw_start {
            let raw = Some(&$me.buffer[$me.raw_start..$me.pos]);

            ($me.token_handler)(Token::Character, raw);
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
