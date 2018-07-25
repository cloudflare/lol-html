#[macro_use]
mod actions;

macro_rules! state_transition {
    ( | $me:tt |> reconsume in $state:ident ) => {
        $me.pos -= 1;
        state_transition!(| $me |> --> $state);
    };

    ( | $me:tt |> --> $state:ident ) => {
        $me.state = Tokenizer::$state;
        $me.state_enter = true;
        return;
    };
}

macro_rules! action_list {
    ( | $me:tt |> $action:tt; $($rest:tt)* ) => {
        action!(| $me |> $action);
        action_list!(| $me |> $($rest)*);
    };

    // NOTE: state transition should always be in the end of the action list
    ( | $me:tt |> $($transition:tt)+ ) => ( state_transition!(| $me |> $($transition)+); );

    // NOTE: end of the action list
    ( | $me:tt |> ) => ();
}

macro_rules! states {
    ( $($states:tt)+ ) => {
        impl<'t, H: FnMut(&Token)> Tokenizer<'t, H> {
           state!($($states)+);
        }
    };
}

macro_rules! enter_actions {
    ( | $me:tt |> $($actions:tt)+) => {
        if $me.state_enter {
            action_list!(|$me|> $($actions)*);
            $me.state_enter = false;
        }
    };

    // NOTE: don't generate any code for the empty action list
    ( | $me:tt |> ) => ();
}

macro_rules! state {
    ( $name:ident { $($arms:tt)* } $($rest:tt)* ) => ( state!($name <-- () { $($arms)* } $($rest)*); );

    // TODO: pub vs private states
    ( $name:ident <-- ( $($enter_actions:tt)* ) { $($arms:tt)* } $($rest:tt)* ) => {
        pub fn $name(&mut self, ch: Option<u8>) {
            enter_actions!(|self|> $($enter_actions)*);
            state_arms!(|self, ch|> $($arms)*);
        }

        state!($($rest)*);
    };

    // NOTE: end of the state list
    () => ();
}

macro_rules! arm_pattern {
    ( ascii_lo ) => ( Some(b'a'...b'z') );
    ( ascii_up ) => ( Some(b'A'...b'Z') );
    ( eof ) => ( None );

    ( $pat:pat ) => ( Some($pat) );
}

macro_rules! state_arms {
    ( | $me:tt, $ch:ident |> $( $pat:tt $(|$pat_cont:tt)*  => ( $($actions:tt)* ) )* ) => {
        match $ch {
            $(
                arm_pattern!($pat) $(| arm_pattern!($pat_cont))* => {
                    action_list!(|$me|> $($actions)*);
                }
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
