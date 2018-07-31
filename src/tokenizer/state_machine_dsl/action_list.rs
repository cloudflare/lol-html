macro_rules! action_list {
    ( | $self:ident |> $action:tt; $($rest:tt)* ) => {
        debug!(@trace_actions $action);
        action!(| $self |> $action);
        action_list!(| $self |> $($rest)*);
    };

    // NOTE: state transition should always be in the end of the action list
    ( | $self:ident |> $($transition:tt)+ ) => {
        debug!(@trace_actions $($transition)+);
        state_transition_action!(| $self |> $($transition)+);
    };

    // NOTE: end of the action list
    ( | $self:ident |> ) => ();


    // State enter action list
    //--------------------------------------------------------------------
    ( @state_enter | $self:ident |> $($actions:tt)+ ) => {
        if $self.state_enter {
            action_list!(|$self|> $($actions)*);
            $self.state_enter = false;
        }
    };

    // NOTE: don't generate any code for the empty action list
    ( @state_enter | $self:ident |> ) => ();
}
