#[macro_use]
mod actions;

macro_rules! action_list {
    (| $me: tt |> $action:tt; $($rest:tt)* ) => {
        action!(| $me |> $action);
        action_list!(| $me |> $($rest)*);
    };

    // NOTE: end of the action list
    (| $me: tt |> ) => (());
}

// TODO: pub vs private
macro_rules! states {
    ( $($name: ident { $($body:tt)* })* ) => {
        impl<'t, H: FnMut(&Token)> Tokenizer<'t, H> {
           $(pub fn $name(&mut self, ch: Option<u8>) {
               state_body!(|self, ch|> $($body)*);
           })*
        }
    };
}

macro_rules! state_body {
    ( | $me: tt, $ch: ident |> on_enter ( $($actions:tt)+ ) $($rest:tt)+ ) => {
        if $me.state_enter {
            action_list!(|$me|> $($actions)*);
            $me.state_enter = false;
        }

        state_body!(| $me, $ch |> $($rest)*);
    };

    ( | $me: tt, $ch: ident |>
        >ch ( $($ch_actions:tt)+ )
        >eof ( $($eof_actions:tt)+ )
    ) => {
        match $ch {
            Some(ch) => { action_list!(|$me|> $($ch_actions)*); }
            None => { action_list!(|$me|> $($eof_actions)*); }
        };
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
