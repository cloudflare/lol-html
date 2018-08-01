macro_rules! arm_pattern {
    ( | $cb_args:tt |>
         alpha => $actions:tt
    ) => {
        state_body!(@callback | $cb_args |> Some(b'a'...b'z') | Some(b'A'...b'Z') => $actions);
    };

    ( | $cb_args:tt |>
        whitespace => $actions:tt
    ) => {
        state_body!(@callback | $cb_args |>
            Some(b' ') | Some(b'\n') | Some(b'r') | Some(b'\t') | Some(b'\x0C') => $actions
        );
    };

    ( | [ [$self:tt, $ch:tt ], $($rest_cb_args:tt)+ ] |>
        closing_quote => $actions:tt
    ) => {
        state_body!(@callback | [ [$self, $ch], $($rest_cb_args)+ ] |>
            Some(ch) if ch == $self.closing_quote => $actions
        );
    };

    ( | $cb_args:tt |>
        eof => $actions:tt
    ) => {
        state_body!(@callback | $cb_args |> None => $actions);
    };

    ( | $cb_args:tt |> $pat:pat => $actions:tt ) => {
        state_body!(@callback | $cb_args |> Some($pat) => $actions);
    };
}
