define_state_group!(rawtext_states_group = {

    rawtext_state {
        >ch ( emit_eof; )
        >eof ( emit_eof; )
    }

});
