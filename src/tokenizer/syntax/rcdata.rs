define_state_group!(rcdata_states_group = {

    pub rcdata_state {
        // TODO
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});
