macro_rules! action {
    ( | $me:tt |> emit_eof ) => {
        ($me.token_handler)(&Token::Eof);
        $me.finished = true;
    };

    ( | $me:tt |> emit_char ) => {
        if $me.pos > $me.token_start {
            let chars = BufferSlice::from(&$me.buffer[$me.token_start..$me.pos]);

            ($me.token_handler)(&Token::Character(chars));
        }
    };

    ( | $me:tt |> mark_token_start ) => {
        $me.token_start = $me.pos;
    };
}
