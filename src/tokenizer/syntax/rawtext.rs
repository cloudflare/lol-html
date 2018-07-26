define_state_group!(rawtext_states_group = {

    pub rawtext_state {
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});
