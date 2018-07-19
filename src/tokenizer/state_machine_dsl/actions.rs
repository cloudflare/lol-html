macro_rules! action {
    ( | $me:tt |> emit_eof ) => {
        ($me.token_handler)(&Token::Eof);
        $me.finished = true;
    };
}
