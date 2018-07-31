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
            debug!(@trace_char ch);
            action_list!(@state_enter |self|> $($($enter_actions)*)*);
            state_body!(| [self, ch] |> $($arms)*);
        }

        state!($($rest)*);
    };


    // NOTE: end of the state list
    () => ();
}
