macro_rules! action {
    (| $self:tt, $input:ident, $ch:ident | > $action_fn:ident ? $($args:expr),* ) => {
        let loop_directive = $self.$action_fn($input, $ch $(,$args),*)?;

        match loop_directive {
            ParsingLoopDirective::None => (),
            _ => {
                return Ok(loop_directive);
            },
        }
    };

    (| $self:tt, $input:ident, $ch:ident | > $action_fn:ident $($args:expr),* ) => {
        $self.$action_fn($input, $ch $(,$args),*);
    };

    ( @state_transition | $self:tt | > reconsume in $state:ident) => {
        $self.input_cursor().unconsume_ch();
        action!(@state_transition | $self | > --> $state);
    };

    ( @state_transition | $self:tt | > - -> $state:ident) => {
        $self.switch_state(Self::$state);

        return Ok(ParsingLoopDirective::Continue);
    };
}
