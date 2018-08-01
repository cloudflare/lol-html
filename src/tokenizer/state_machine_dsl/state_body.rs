macro_rules! state_body {
    ( | $scope_vars:tt |> $($arms:tt)+ ) => {
        state_body!(@map_arms | $scope_vars |> [$($arms)+], [])
    };


    // Recursively expand each arm's pattern
    //--------------------------------------------------------------------
    ( @map_arms
        | $scope_vars:tt |>
        [ $pat:tt => ( $($actions:tt)* ) $($rest:tt)* ], [ $($expanded:tt)* ]
    ) => {
        state_body!(@arm_pat | [ $scope_vars, [$($rest)*], [$($expanded)*] ]|>
            $pat => ( $($actions)* )
        )
    };

    ( @map_arms
        | $scope_vars:tt |>
        [], [$($expanded:tt)*]
    ) => {
        state_body!(@match_block |$scope_vars|> $($expanded)*);
    };


    // Callback for the expand_arm_pattern
    //--------------------------------------------------------------------
    ( @callback
        | [ $scope_vars:tt, [$($pending:tt)*], [$($expanded:tt)*] ] |>
        $($expanded_arm:tt)*
    ) => {
        state_body!(@map_arms | $scope_vars |> [$($pending)*], [$($expanded)* $($expanded_arm)*])
    };


    // Arm patterns
    //--------------------------------------------------------------------
    ( @arm_pat | $cb_args:tt |> alpha => $actions:tt ) => {
        state_body!(@callback | $cb_args |> Some(b'a'...b'z') | Some(b'A'...b'Z') => $actions);
    };

    ( @arm_pat | $cb_args:tt |> whitespace => $actions:tt ) => {
        state_body!(@callback | $cb_args |>
            Some(b' ') | Some(b'\n') | Some(b'r') | Some(b'\t') | Some(b'\x0C') => $actions
        );
    };

    ( @arm_pat | [ [$self:tt, $ch:ident ], $($rest_cb_args:tt)+ ] |>
        closing_quote => $actions:tt
    ) => {
        state_body!(@callback | [ [$self, $ch], $($rest_cb_args)+ ] |>
            Some(ch) if ch == $self.closing_quote => $actions
        );
    };

    ( @arm_pat | $cb_args:tt |> eof => $actions:tt ) => {
        state_body!(@callback | $cb_args |> None => $actions);
    };

    ( @arm_pat | $cb_args:tt |> $pat:pat => $actions:tt ) => {
        state_body!(@callback | $cb_args |> Some($pat) => $actions);
    };


    // Character match block
    //--------------------------------------------------------------------
    ( @match_block
        | [ $self:tt, $ch:ident ] |>
        $( $pat:pat $(|$pat_cont:pat)* $(if $pat_expr:expr)* => ( $($actions:tt)* ) )*
    ) => {
        match $ch {
            $(
                $pat $(| $pat_cont)* $(if $pat_expr)* => { action_list!(|$self|> $($actions)*); }
            )*
        }
    };
}
