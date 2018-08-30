macro_rules! state_transition_action {
    (| $self:tt | > reconsume in $state:ident) => {
        $self.pos -= 1;
        action_helper!(@switch_state |$self|> Tokenizer::$state);
    };

    (| $self:tt | > - -> $state:ident) => {
        action_helper!(@switch_state |$self|> Tokenizer::$state);
    };
}
