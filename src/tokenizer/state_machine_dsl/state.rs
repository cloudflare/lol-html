macro_rules! state {
    (
        $vis:vis $name:ident $(<-- ( $($enter_actions:tt)* ))* {
            $($arms:tt)*
        }

        $($rest:tt)*
    ) => {
        $vis fn $name(&mut self, input: &Chunk) -> Result<ParsingLoopDirective, Error> {
            // NOTE: clippy complains about some states that break the loop in each match arm
            #[cfg_attr(feature = "cargo-clippy", allow(never_loop))]
            loop {
                let ch = input!(@consume_ch self, input);

                state_body!(|[self, input, ch]|> [$($arms)*], [$($($enter_actions)*)*]);
            }

            // NOTE: this can be unreachable if all state body
            // arms expand into state transitions.
            #[allow(unreachable_code)] { return Ok(ParsingLoopDirective::Continue); }
        }

        state!($($rest)*);
    };

    // NOTE: end of the state list
    () => ();
}
