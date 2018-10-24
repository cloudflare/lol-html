macro_rules! state_transition_action {
    (| $self:tt, $input:ident | > reconsume in $state:ident) => {
        input!(@unconsume_ch $self);
        action_helper!(@switch_state |$self|> Tokenizer::$state);
    };

    (| $self:tt, $input:ident | > - -> $state:ident) => {
        action_helper!(@switch_state |$self|> Tokenizer::$state);
    };
}
