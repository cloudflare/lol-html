define_state_group!(rcdata_states_group = {

    rcdata_state {
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});
