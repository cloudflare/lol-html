#[macro_use]
mod actions;

macro_rules! action_list {
    // NOTE: state transition should always be in the end of the action list
    (| $me: tt |> --> $state:ident) => {
        $me.state = Tokenizer::$state;
        $me.state_enter = true;
        return;
    };

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

macro_rules! character_handler {
    ( | $me: tt |> ( $($actions:tt)+ ) ) => {
        action_list!(|$me|> $($actions)*);
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

    // NOTE: with this macro we enforce that all states should
    // handle both characters and EOF explicitly. To avoid parser
    // ambiguity EOF always should be first, since it always has
    // its actions enclosed in braces, whereas for character it
    // either brace-enclosed list of actions or list of match arms.
    ( | $me: tt, $ch: ident |>
        >eof ( $($eof_actions:tt)+ )
        >ch $($ch_handler:tt)+
    ) => {
        match $ch {
            Some(ch) => { character_handler!(|$me|> $($ch_handler)*); }
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
