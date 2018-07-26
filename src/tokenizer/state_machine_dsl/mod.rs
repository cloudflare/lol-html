#[macro_use]
mod actions;

macro_rules! state_transition {
    ( | $me:ident |> reconsume in $state:ident ) => {
        $me.pos -= 1;
        state_transition!(| $me |> --> $state);
    };

    ( | $me:ident |> --> $state:ident ) => {
        $me.state = Tokenizer::$state;
        $me.state_enter = true;
        return;
    };
}

macro_rules! action_list {
    ( | $me:ident |> $action:tt; $($rest:tt)* ) => {
        action!(| $me |> $action);
        action_list!(| $me |> $($rest)*);
    };

    // NOTE: state transition should always be in the end of the action list
    ( | $me:ident |> $($transition:tt)+ ) => ( state_transition!(| $me |> $($transition)+); );

    // NOTE: end of the action list
    ( | $me:ident |> ) => ();
}

macro_rules! states {
    ( $($states:tt)+ ) => {
        impl<'t, H: FnMut(&Token)> Tokenizer<'t, H> {
           state!($($states)+);
        }
    };
}

macro_rules! enter_actions {
    ( | $me:ident |> $($actions:tt)+) => {
        if $me.state_enter {
            action_list!(|$me|> $($actions)*);
            $me.state_enter = false;
        }
    };

    // NOTE: don't generate any code for the empty action list
    ( | $me:ident |> ) => ();
}

macro_rules! state {
    // NOTE: wrap optional visibility modifier in `[]` to avoid
    // local ambiguity with the state name.
    ( pub $($rest:tt)* ) => ( state!([pub] $($rest)*); );

    ( $([ $vis:ident ])* $name:ident { $($arms:tt)* } $($rest:tt)* ) => {
        state!($([$vis])* $name <-- () { $($arms)* } $($rest)*);
    };

    ( $([ $vis:ident ])* $name:ident <-- ( $($enter_actions:tt)* ) { $($arms:tt)* } $($rest:tt)* ) => {
        $($vis)* fn $name(&mut self, ch: Option<u8>) {
            enter_actions!(|self|> $($enter_actions)*);
            state_body!(| [self, ch] |> $($arms)*);
        }

        state!($($rest)*);
    };

    // NOTE: end of the state list
    () => ();
}

macro_rules! expand_arm_pattern {
    ( | $cb_args:tt |> alpha => $actions:tt ) => {
        state_body!(@callback | $cb_args |> Some(b'a'...b'z') | Some(b'A'...b'Z') => $actions);
    };

    ( | $cb_args:tt |> eof => $actions:tt ) => {
        state_body!(@callback | $cb_args |> None => $actions);
    };

    ( | $cb_args:tt |> $pat:pat => $actions:tt ) => {
        state_body!(@callback | $cb_args |> Some($pat) => $actions);
    };
}

macro_rules! state_body {
    ( | $scope_vars:tt |> $($arms:tt)+ ) => {
        state_body!(@map_arms | $scope_vars |> [$($arms)+], [])
    };

    // NOTE: recursively expand each arm
    ( @map_arms
        | $scope_vars:tt |>
        [ $pat:tt => ( $($actions:tt)* ) $($rest:tt)* ], [ $($expanded:tt)* ]
    ) => {
        expand_arm_pattern!(
            | [ $scope_vars, [$($rest)*], [$($expanded)*] ]|>
            $pat => ( $($actions)* )
        )
    };

    // NOTE: end of iteration
    ( @map_arms
        | $scope_vars:tt |>
        [], [$($expanded:tt)*]
    ) => {
        state_body!(@match_block |$scope_vars|> $($expanded)*);
    };

    // NOTE: callback for the expand_arm_pattern!
    ( @callback
        | [ $scope_vars:tt, [$($pending:tt)*], [$($expanded:tt)*] ] |>
        $($expanded_arm:tt)*
    ) => {
        state_body!(@map_arms | $scope_vars |> [$($pending)*], [$($expanded)* $($expanded_arm)*])
    };

    ( @match_block
        | [ $me:ident, $ch:ident ] |>
        $( $pat:pat $(|$pat_cont:pat)* => ( $($actions:tt)* ) )*
    ) => {
        match $ch {
            $(
                $pat $(| $pat_cont)* => { action_list!(|$me|> $($actions)*); }
            )*
        }
    };
}

macro_rules! define_state_group {
    ( $name:ident = { $($states:tt)+ } ) => {
        macro_rules! $name {
            () => {
                states!($($states)*);
            };
        }
    };
}
