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
        $($vis)* fn $name(&mut self, ch: Option<u8>) -> Result<(), TokenizerBailoutReason> {
            trace!(@chars ch);
            state_body!(| [self, ch] |> [$($arms)*], [$($($enter_actions)*)*]);

            // NOTE: this can be unreachable if all state body
            // arms expand into state transitions.
            #[allow(unreachable_code)] { return Ok(()); }
        }

        state!($($rest)*);
    };

    // NOTE: end of the state list
    () => ();
}
