macro_rules! action {
    ( | $me:ident |> emit_eof ) => {
        ($me.token_handler)(&Token::Eof);
        $me.finished = true;
    };

    ( | $me:ident |> emit_char ) => {
        if $me.pos > $me.token_start {
            let chars = BufferSlice::from(&$me.buffer[$me.token_start..$me.pos]);

            ($me.token_handler)(&Token::Character(chars));
        }
    };

    ( | $me:ident |> mark_token_start ) => {
        $me.token_start = $me.pos;
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
