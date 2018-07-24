macro_rules! action {
    ( | $me:tt |> emit_eof ) => {
        ($me.token_handler)(&Token::Eof);
        $me.finished = true;
    };

    ( | $me:tt |> emit_char ) => {
        if $me.pos > $me.token_start {
            let chars = &$me.buffer[$me.token_start..$me.pos];

            ($me.token_handler)(&Token::Character(chars));
        }
    };

    ( | $me:tt |> create_char ) => {
        $me.token_start = $me.pos;
    };
}
