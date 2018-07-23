define_state_group!(plaintext_states_group = {

    plaintext_state {
        >eof ( emit_eof; )
        >ch ( emit_eof; )
    }

});
