define_state_group!(end_tag_states_group = {

    end_tag_open_state {
        // TODO
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});
