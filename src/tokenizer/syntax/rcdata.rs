define_state_group!(rcdata_states_group = {

    pub rcdata_state <-- ( start_raw; ) {
        // TODO
        eof => ( emit_chars; emit_eof; )
        _   => ()
    }

});
