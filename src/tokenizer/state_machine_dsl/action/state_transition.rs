macro_rules! state_transition_action {
    ( | $self:tt |> reconsume in $state:ident ) => {
        $self.pos -= 1;
        state_transition_action!(| $self |> --> $state);
    };

    ( | $self:tt |> --> $state:ident ) => {
        $self.state = Tokenizer::$state;
        $self.state_enter = true;
        return;
    };
}
