macro_rules! action_helper {
    ( @emit_lex_unit_with_raw_inclusive | $self:tt |> $token:expr ) => {
        action_helper!(@emit_lex_unit_with_raw |$self|> $token, $self.pos + 1 )
    };

    ( @emit_lex_unit_with_raw_exclusive | $self:tt |> $token:expr ) => {
        action_helper!(@emit_lex_unit_with_raw |$self|> $token, $self.pos )
    };

    ( @emit_lex_unit_with_raw | $self:tt |> $token:expr, $end:expr ) => ({
        trace!(@raw $self, $end);

        action_helper!(@emit_lex_unit |$self|>
            $token,
            Some(&$self.input_chunk[$self.raw_start..$end])
        )
    });

    ( @emit_lex_unit | $self:tt |> $token:expr, $raw:expr ) => ({
        let lex_unit = LexUnit {
            token_view: $token,
            raw: $raw,
        };

        $self.lex_unit_handler.handle(&lex_unit);

        lex_unit
    });

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
        if let Some(AttributeView { ref mut $part, .. }) = $self.current_attr {
            action_helper!(@set_token_part_range |$self|> $part);
        }
    };

    ( @update_tag_part | $self:tt |> $part:ident, $action:block ) => {
        match $self.current_token {
            Some(TokenView::StartTag { ref mut $part, .. }) |
            Some(TokenView::EndTag { ref mut $part, .. }) => $action
            _ => unreachable!("Current token should always be a start or an end tag at this point")
        }
    };

    ( @switch_state | $self:tt |> $state:expr ) => {
        $self.state = $state;
        $self.state_enter = true;
        return Ok(());
    };

    ( @notify_text_parsing_mode_change | $self:tt |> $mode:expr ) => {
        #[cfg(feature = "testing_api")]
        {
            if let Some(ref mut text_parsing_mode_change_handler) =
                $self.text_parsing_mode_change_handler
            {
                text_parsing_mode_change_handler.handle(TextParsingModeSnapshot {
                    mode: $mode,
                    last_start_tag_name_hash: $self.last_start_tag_name_hash,
                });
            }
        }
    };
}
