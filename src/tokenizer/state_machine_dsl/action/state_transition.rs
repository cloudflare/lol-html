macro_rules! state_transition_action {
    (| $self:tt, $input_chunk:ident | > reconsume in $state:ident) => {
        input!(@unconsume_ch $self);
        action_helper!(@switch_state |$self|> Tokenizer::$state);
    };

    (| $self:tt, $input_chunk:ident | > - -> $state:ident) => {
        action_helper!(@switch_state |$self|> Tokenizer::$state);
    };
}
