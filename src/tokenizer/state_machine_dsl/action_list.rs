macro_rules! action_list {
    ( | $self:tt, $input:ident, $ch:ident |>
        if $cond:ident
            ( $($if_actions:tt)* )
        else
            ( $($else_actions:tt)* )
    ) => {
        if $self.$cond($ch) {
            action_list!(| $self, $input, $ch |> $($if_actions)*);
        } else {
            action_list!(| $self, $input, $ch |> $($else_actions)*);
        }
    };

    ( | $self:tt, $input:ident, $ch:ident |> { $($code_block:tt)* } ) => ( $($code_block)* );

    ( | $self:tt, $input:ident, $ch:ident |> $action:ident $($args:expr),*; $($rest:tt)* ) => {
        trace!(@actions $action $($args:expr)*);
        action!(| $self, $input, $ch |> $action $($args),*);
        action_list!(| $self, $input, $ch |> $($rest)*);
    };

     ( | $self:tt, $input:ident, $ch:ident |> $action:ident ? $($args:expr),*; $($rest:tt)* ) => {
        trace!(@actions $action $($args:expr)*);
        action!(| $self, $input, $ch |> $action ? $($args),*);
        action_list!(| $self, $input, $ch |> $($rest)*);
    };

    // NOTE: state transition should always be in the end of the action list
    ( | $self:tt, $input:ident, $ch:ident |> $($transition:tt)+ ) => {
        trace!(@actions $($transition)+);
        action!(@state_transition | $self |> $($transition)+);
    };

    // NOTE: end of the action list
    ( | $self:tt, $input:ident, $ch:ident |> ) => ();


    // State enter action list
    //--------------------------------------------------------------------
    ( @state_enter | $self:tt, $input:ident, $ch:ident |> $($actions:tt)+ ) => {
        if $self.is_state_enter() {
            action_list!(|$self, $input, $ch|> $($actions)*);
            $self.set_is_state_enter(false);
        }
    };

    // NOTE: don't generate any code for the empty action list
    ( @state_enter | $self:tt, $input:ident, $ch:ident |> ) => ();
}
