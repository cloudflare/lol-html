define_state_group!(plaintext_states_group = {

    pub plaintext_state <-- ( start_raw; ) {
        eof => ( emit_chars; emit_eof; )
        _   => ()
    }

});
