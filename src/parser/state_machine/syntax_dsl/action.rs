macro_rules! action {
    (| $self:tt, $input:ident | > $action_fn:ident ? $($args:expr),* ) => {
        let loop_directive = $self.$action_fn($input $(,$args),*)?;

        match loop_directive {
            ParsingLoopDirective::None => (),
            _ => {
                return Ok(loop_directive);
            },
        }
    };

    (| $self:tt, $input:ident | > $action_fn:ident $($args:expr),* ) => {
        $self.$action_fn($input $(,$args),*);
    };

    ( @state_transition | $self:tt, $input:ident | > reconsume in $state:ident) => {
        $self.unconsume_ch();
        action!(@state_transition | $self, $input | > --> $state);
    };

    ( @state_transition | $self:tt, $input:ident | > - -> $state:ident) => {
        $self.switch_state(Self::$state);

        return Ok(ParsingLoopDirective::Continue);
    };
}
