#[macro_use]
mod ch_sequence;

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
            Some(b' ') | Some(b'\n') | Some(b'\r') | Some(b'\t') | Some(b'\x0C') => $actions
        );
    };

    ( | [ [$self:tt, $input_chunk:ident, $ch:ident ], $($rest_cb_args:tt)+ ] |>
        closing_quote => $actions:tt
    ) => {
        state_body!(@callback | [ [$self, $input_chunk, $ch], $($rest_cb_args)+ ] |>
            Some(ch) if ch == $self.closing_quote => $actions
        );
    };

    ( | $cb_args:tt |>
        eof => $actions:tt
    ) => {
        state_body!(@callback | $cb_args |> None => $actions);
    };

    ( | [ $scope_vars:tt, $($rest_cb_args:tt)+ ] |>
        [ $seq_pat:tt $(; $case_mod:ident)* ] => $actions:tt
    ) => {
        // NOTE: character sequence arm should be expanded in
        // place before we hit the character match block.
        ch_sequence_arm_pattern!(|$scope_vars|> $seq_pat, $actions, $($case_mod)* );
        state_body!(@callback | [ $scope_vars, $($rest_cb_args)+ ] |>);
    };

    ( | $cb_args:tt |> $pat:pat => $actions:tt ) => {
        state_body!(@callback | $cb_args |> Some($pat) => $actions);
    };
}
