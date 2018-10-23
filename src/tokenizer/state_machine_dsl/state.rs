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
        $($vis)* fn $name(
            &mut self,
            input_chunk: &mut IterableChunk,
            ch: Option<u8>
        ) -> Result<ParsingLoopDirective, Error> {
            trace!(@chars ch);
            state_body!(|[self, input_chunk, ch]|> [$($arms)*], [$($($enter_actions)*)*]);

            // NOTE: this can be unreachable if all state body
            // arms expand into state transitions.
            #[allow(unreachable_code)] { return Ok(ParsingLoopDirective::Continue); }
        }

        state!($($rest)*);
    };

    // NOTE: end of the state list
    () => ();
}
