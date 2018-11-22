define_state_group!(plaintext_states_group = {

    plaintext_state {
        eoc => ( emit_chars; )
        eof => ( emit_chars; emit_eof; )
        _   => ()
    }

});
