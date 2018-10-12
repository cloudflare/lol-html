macro_rules! action_list {
    ( | $self:tt, $input_chunk:ident, $ch:ident |>
        if $cond:ident ( $($if_actions:tt)* ) else ( $($else_actions:tt)* )
    ) => {
        if condition!(|$self|> $cond) {
            action_list!(| $self, $input_chunk, $ch |> $($if_actions)*);
        } else {
            action_list!(| $self, $input_chunk, $ch |> $($else_actions)*);
        }
    };

    ( | $self:tt, $input_chunk:ident, $ch:ident |> $action:tt $($args:expr)*; $($rest:tt)* ) => {
        trace!(@actions $action $($args:expr)*);
        action!(| $self, $input_chunk, $ch |> $action $($args)*);
        action_list!(| $self, $input_chunk, $ch |> $($rest)*);
    };

    // NOTE: state transition should always be in the end of the action list
    ( | $self:tt, $input_chunk:ident, $ch:ident |> $($transition:tt)+ ) => {
        trace!(@actions $($transition)+);
        state_transition_action!(| $self |> $($transition)+);
    };

    // NOTE: end of the action list
    ( | $self:tt, $input_chunk:ident, $ch:ident |> ) => ();


    // State enter action list
    //--------------------------------------------------------------------
    ( @state_enter | $self:tt, $input_chunk:ident, $ch:ident |> $($actions:tt)+ ) => {
        if $self.state_enter {
            action_list!(|$self, $input_chunk, $ch|> $($actions)*);
            $self.state_enter = false;
        }
    };

    // NOTE: don't generate any code for the empty action list
    ( @state_enter | $self:tt, $input_chunk:ident, $ch:ident |> ) => ();
}
