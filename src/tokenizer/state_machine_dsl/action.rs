macro_rules! action {
    (| $self:tt, $input:ident, $ch:ident | > $action_fn:ident ? $($args:expr),* ) => {
        if let Some(loop_directive) = $self.$action_fn($input, $ch $(,$args),*)? {
            return Ok(loop_directive);
        }
    };

    (| $self:tt, $input:ident, $ch:ident | > $action_fn:ident $($args:expr),* ) => {
        $self.$action_fn($input, $ch $(,$args),*);
    };

    ( @state_transition | $self:tt | > reconsume in $state:ident) => {
        $self.get_input_cursor().unconsume_ch();
        action!(@state_transition | $self | > --> $state);
    };

    ( @state_transition | $self:tt | > - -> $state:ident) => {
        $self.switch_state(Self::$state);

        return Ok(ParsingLoopDirective::Continue);
    };
}
