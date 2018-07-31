macro_rules! action_helper {
    ( @emit_lex_result_with_raw_inclusive | $me:ident |> $token:expr ) => {
        debug!(@trace_raw $me, $me.pos + 1);
        action_helper!(@emit_lex_result |$me|> $token, Some(&$me.buffer[$me.raw_start..=$me.pos]));
    };

    ( @emit_lex_result_with_raw_exclusive | $me:ident |> $token:expr ) => {
        debug!(@trace_raw $me, $me.pos);
        action_helper!(@emit_lex_result |$me|> $token, Some(&$me.buffer[$me.raw_start..$me.pos]));
    };

    ( @emit_lex_result | $me:ident |> $token:expr, $raw:expr ) => {
        let res = LexResult {
            shallow_token: $token,
            raw: $raw,
        };

        ($me.token_handler)(res);
    };

    ( @set_token_part_range | $me:ident |> $part:ident ) => {
        $part.start = $me.token_part_start;
        $part.end = $me.pos - $me.raw_start;
    };
}
