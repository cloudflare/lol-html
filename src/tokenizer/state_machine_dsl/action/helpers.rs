macro_rules! action_helper {
    ( @emit_textual_token | $me:ident |> $ty:ident ) => {
        if $me.pos > $me.raw_start {
            let res = LexResult {
                shallow_token: ShallowToken::$ty,
                raw: action_helper!(@get_raw |$me|> ),
            };

            ($me.token_handler)(res);
        }
    };

    ( @get_raw | $me:ident |> ) => {
        Some(&$me.buffer[$me.raw_start..$me.pos])
    };

    ( @emit_lex_result | $me:ident|> $token:expr ) => {
        action_helper!(@emit_lex_result |$me|> $token, action_helper!(@get_raw |$me|>))
    };

    ( @emit_lex_result | $me:ident |> $token:expr, $raw:expr ) => {
        ($me.token_handler)(LexResult {
            shallow_token: $token,
            raw: $raw,
        });
    };

    ( @set_token_part_range | $me:ident |> $part:ident ) => {
        $part.start = $me.token_part_start;
        $part.end = $me.pos - $me.raw_start;
    };
}
