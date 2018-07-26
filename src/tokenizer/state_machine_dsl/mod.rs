#[macro_use]
mod action;

#[macro_use]
mod action_list;

#[macro_use]
mod state_body;

macro_rules! state_group {
    ( $($states:tt)+ ) => {
        impl<'t, H: FnMut(&Token)> Tokenizer<'t, H> {
           state!($($states)+);
        }
    };
}

macro_rules! state {
    // NOTE: wrap optional visibility modifier in `[]` to avoid
    // local ambiguity with the state name.
    ( pub $($rest:tt)* ) => ( state!([pub] $($rest)*); );

    (
        $([ $vis:ident ])* $name:ident $(<-- ( $($enter_actions:tt)* ))* {
            $($arms:tt)*
        }

        $($rest:tt)*
    ) => {
        $($vis)* fn $name(&mut self, ch: Option<u8>) {
            action_list!(@state_enter |self|> $($($enter_actions)*)*);
            state_body!(| [self, ch] |> $($arms)*);
        }

        state!($($rest)*);
    };


    // NOTE: end of the state list
    () => ();
}

macro_rules! define_state_group {
    ( $name:ident = { $($states:tt)+ } ) => {
        macro_rules! $name {
            () => {
                state_group!($($states)*);
            };
        }
    };
}
