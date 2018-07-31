macro_rules! action_list {
    ( | $me:ident |> $action:tt; $($rest:tt)* ) => {
        debug!(@trace_actions $action);
        action!(| $me |> $action);
        action_list!(| $me |> $($rest)*);
    };

    // NOTE: state transition should always be in the end of the action list
    ( | $me:ident |> $($transition:tt)+ ) => {
        debug!(@trace_actions $($transition)+);
        state_transition_action!(| $me |> $($transition)+);
    };

    // NOTE: end of the action list
    ( | $me:ident |> ) => ();


    // State enter action list
    //--------------------------------------------------------------------
    ( @state_enter | $me:ident |> $($actions:tt)+ ) => {
        if $me.state_enter {
            action_list!(|$me|> $($actions)*);
            $me.state_enter = false;
        }
    };

    // NOTE: don't generate any code for the empty action list
    ( @state_enter | $me:ident |> ) => ();
}
