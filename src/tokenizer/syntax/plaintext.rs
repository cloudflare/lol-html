define_state_group!(plaintext_states_group = {

    pub plaintext_state {
        // TODO
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});
