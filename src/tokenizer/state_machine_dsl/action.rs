macro_rules! action {
    ( | $me:ident |> emit_eof ) => {
        ($me.token_handler)(Token::Eof);
        $me.finished = true;
    };

    ( | $me:ident |> emit_chars ) => {
        if $me.pos > $me.slice_start {
            let chars = BufferSlice::from(&$me.buffer[$me.slice_start..$me.pos]);

            ($me.token_handler)(Token::Character(chars));
        }
    };

    ( | $me:ident |> start_slice ) => {
        $me.slice_start = $me.pos;
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
