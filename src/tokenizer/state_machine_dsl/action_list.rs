macro_rules! action_list {
    ( | $self:tt, $ch:ident |>
        if $cond:ident ( $($if_actions:tt)* ) else ( $($else_actions:tt)* )
    ) => {
        if condition!(|$self|> $cond) {
            action_list!(| $self, $ch |> $($if_actions)*);
        } else {
            action_list!(| $self, $ch |> $($else_actions)*);
        }
    };

    ( | $self:tt, $ch:ident |> $action:tt $($args:expr)*; $($rest:tt)* ) => {
        debug!(@trace_actions $action $($args:expr)*);
        action!(| $self, $ch |> $action $($args)*);
        action_list!(| $self, $ch |> $($rest)*);
    };

    // NOTE: state transition should always be in the end of the action list
    ( | $self:tt, $ch:ident |> $($transition:tt)+ ) => {
        debug!(@trace_actions $($transition)+);
        state_transition_action!(| $self |> $($transition)+);
    };

    // NOTE: end of the action list
    ( | $self:tt, $ch:ident |> ) => ();


    // State enter action list
    //--------------------------------------------------------------------
    ( @state_enter | $self:tt, $ch:ident |> $($actions:tt)+ ) => {
        if $self.state_enter {
            action_list!(|$self, $ch|> $($actions)*);
            $self.state_enter = false;
        }
    };

    // NOTE: don't generate any code for the empty action list
    ( @state_enter | $self:tt, $ch:ident |> ) => ();
}
