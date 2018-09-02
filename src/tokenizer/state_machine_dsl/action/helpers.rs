macro_rules! action_helper {
    ( @emit_lex_result_with_raw_inclusive | $self:tt |> $token:expr ) => {
        action_helper!(@emit_lex_result_with_raw |$self|> $token, $self.pos + 1 )
    };

    ( @emit_lex_result_with_raw_exclusive | $self:tt |> $token:expr ) => {
        action_helper!(@emit_lex_result_with_raw |$self|> $token, $self.pos )
    };

    ( @emit_lex_result_with_raw | $self:tt |> $token:expr, $end:expr ) => {
        trace!(@raw $self, $end);

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

        if let Some(state) = $self.lex_res_handler.handle_and_provide_feedback(res) {
            action_helper!(@switch_state |$self|> state);
        }
    };

    ( @set_token_part_range | $self:tt |> $part:ident ) => {
        $part.start = $self.token_part_start;
        $part.end = $self.pos - $self.raw_start;
    };

    ( @set_opt_token_part_range | $self:tt |> $part:ident ) => {
        *$part = Some({
            let mut $part = SliceRange::default();

            action_helper!(@set_token_part_range |$self|> $part);

            $part
        });
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

    ( @update_tag_part | $self:tt |> $part:ident, $action:block ) => {
        match $self.current_token {
            Some(ShallowToken::StartTag { ref mut $part, .. }) |
            Some(ShallowToken::EndTag { ref mut $part, .. }) => $action
            _ => unreachable!("Current token should always be a start or an end tag at this point")
        }
    };

    ( @switch_state | $self:tt |> $state:expr ) => {
        $self.state = $state;
        $self.state_enter = true;
        return;
    };
}
