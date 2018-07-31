macro_rules! state_transition_action {
    ( | $me:ident |> reconsume in $state:ident ) => {
        $me.pos -= 1;
        state_transition_action!(| $me |> --> $state);
    };

    ( | $me:ident |> --> $state:ident ) => {
        $me.state = Tokenizer::$state;
        $me.state_enter = true;
        return;
    };
}
