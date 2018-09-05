macro_rules! emit_tag {
    ( $self:tt ) => {
        let token = $self.current_token.take();

        match token {
            Some(ShallowToken::StartTag { name_hash, .. }) => {
                $self.last_start_tag_name_hash = name_hash;

                emit_tag!(@emit_and_get_feedback |$self|> token, name_hash, get_feedback_for_start_tag_name);
            }
            Some(ShallowToken::EndTag { name_hash, .. }) => {
                emit_tag!(@emit_and_get_feedback |$self|> token, name_hash, get_feedback_for_end_tag_name);
            }
            _ => unreachable!("Token should be a start or an end tag at this point"),
        }
    };

    ( @emit_and_get_feedback | $self:tt | > $token:ident, $name_hash:ident, $get_feedback:ident ) => {
        let mut feedback = $self.tree_builder_simulator.$get_feedback($name_hash);
        let lex_unit = action_helper!(@emit_lex_unit_with_raw_inclusive |$self|> $token);

        emit_tag!(@handle_tree_builder_feedback |$self|> feedback, lex_unit);
    };

    ( @handle_tree_builder_feedback | $self:tt | > $feedback:ident, $lex_unit:ident ) => {
        loop {
            match $feedback {
                TreeBuilderFeedback::Adjust(adjustment) => {
                    emit_tag!(@apply_adjustment |$self|> adjustment);
                    break;
                }
                TreeBuilderFeedback::RequestStartTagToken(reason) => {
                    let token = $lex_unit.as_token().expect("There should be a token at this point");

                    $feedback = $self.tree_builder_simulator.fulfill_start_tag_token_request(token, reason);
                }
                TreeBuilderFeedback::RequestEndTagToken => {
                    let token = $lex_unit.as_token().expect("There should be a token at this point");

                    $feedback = $self.tree_builder_simulator.fulfill_end_tag_token_request(token);
                },
                TreeBuilderFeedback::RequestSelfClosingFlag => {
                    match $lex_unit.shallow_token {
                        Some(ShallowToken::StartTag { self_closing, ..}) => {
                            $feedback = $self.tree_builder_simulator.fulfill_self_closing_flag_request(self_closing);
                        },
                        _ => unreachable!("Token should be a start tag at this point"),
                    }
                }
                TreeBuilderFeedback::None => break,
            }
        }
    };

    ( @apply_adjustment | $self:tt | > $adjustment:ident ) => {
        match $adjustment {
            TokenizerAdjustment::SwitchTextParsingMode(mode) => {
                action_helper!(@notify_text_parsing_mode_change |$self|> mode);
                action_helper!(@switch_state |$self|> mode.into());
            }
            TokenizerAdjustment::SetAllowCdata(allow_cdata) => {
                $self.allow_cdata = allow_cdata;
            }
        }
    };
}
