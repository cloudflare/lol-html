macro_rules! state {
    (
        $vis:vis $name:ident $(<-- ( $($enter_actions:tt)* ))* {
            $($arms:tt)*
        }

        $($rest:tt)*
    ) => {
        $vis fn $name(
            &mut self,
            input: &Chunk,
            ch: Option<u8>
        ) -> Result<ParsingLoopDirective, Error> {
            state_body!(|[self, input, ch]|> [$($arms)*], [$($($enter_actions)*)*]);

            // NOTE: this can be unreachable if all state body
            // arms expand into state transitions.
            #[allow(unreachable_code)] { return Ok(ParsingLoopDirective::Continue); }
        }

        state!($($rest)*);
    };

    // NOTE: end of the state list
    () => ();
}
