define_state_group!(rawtext_states_group = {

    rawtext_state {
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});
