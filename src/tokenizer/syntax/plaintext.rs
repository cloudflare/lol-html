define_state_group!(plaintext_states_group = {

    plaintext_state {
        >ch ( emit_eof; )
        >eof ( emit_eof; )
    }

});
