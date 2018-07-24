#[macro_use]
mod actions;

macro_rules! action_list {
    // NOTE: state transition should always be in the end of the action list
    ( | $me:tt |> --> $state:ident) => {
        $me.state = Tokenizer::$state;
        $me.state_enter = true;
        return;
    };

    ( | $me:tt |> $action:tt; $($rest:tt)* ) => {
        action!(| $me |> $action);
        action_list!(| $me |> $($rest)*);
    };

    // NOTE: end of the action list
    ( | $me:tt |> ) => (());
}

// TODO: pub vs private
macro_rules! states {
    ( $($name:ident { $($body:tt)* })* ) => {
        impl<'t, H: FnMut(&Token)> Tokenizer<'t, H> {
           $(pub fn $name(&mut self, ch: Option<u8>) {
               state_body!(|self, ch|> $($body)*);
           })*
        }
    };
}

macro_rules! arm_pattern {
    ($id:ident) => (arm_pattern!(@maybe_eof $id));
    (@maybe_eof eof) => (None);

    ($pattern:pat) => (Some($pattern));
}

macro_rules! state_body {
    ( | $me:tt, $ch:ident |> $( $pattern:tt => ( $($actions:tt)* ) )* ) => {
        match $ch {
            $(
                arm_pattern!($pattern) => { action_list!(|$me|> $($actions)*); }
            )*
        }
    };
}

macro_rules! define_state_group {
    ($name:ident = { $($states:tt)+ }) => {
        macro_rules! $name {
            () => {
                states!($($states)*);
            };
        }
    };
}
