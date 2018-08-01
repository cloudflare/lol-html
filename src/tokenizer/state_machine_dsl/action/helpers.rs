macro_rules! action_helper {
    ( @emit_lex_result_with_raw_inclusive | $self:tt |> $token:expr ) => {
        action_helper!(@emit_lex_result_with_raw |$self|> $token, $self.pos + 1 )
    };

    ( @emit_lex_result_with_raw_exclusive | $self:tt |> $token:expr ) => {
        action_helper!(@emit_lex_result_with_raw |$self|> $token, $self.pos )
    };

    ( @emit_lex_result_with_raw | $self:tt |> $token:expr, $end:expr ) => {
        debug!(@trace_raw $self, $end);

        action_helper!(@emit_lex_result |$self|>
            $token,
            Some(&$self.buffer[$self.raw_start..$end])
        );
    };

    ( @emit_lex_result | $self:tt |> $token:expr, $raw:expr ) => {
        let res = LexResult {
            shallow_token: $token,
            raw: $raw,
        };

        ($self.token_handler)(res);
    };

    ( @set_token_part_range | $self:tt |> $part:ident ) => {
        $part.start = $self.token_part_start;
        $part.end = $self.pos - $self.raw_start;
    };

    ( @finish_attr_part | $self:tt |> $part:ident ) => {
        match $self.current_attr {
            Some(ShallowAttribute { ref mut $part, .. }) => {
                action_helper!(@set_token_part_range |$self|> $part);
            }
            // NOTE: end tag case
            None => ()
        }
    };
}
