define_state_group!(script_data_double_escaped_states_group = {

    script_data_double_escape_start_state {
        whitespace => ( --> script_data_double_escaped_state )
        b'/'       => ( --> script_data_double_escaped_state )
        b'>'       => ( --> script_data_double_escaped_state )
        eof        => ( emit_chars; emit_eof; )
        _          => ( reconsume in script_data_escaped_state )
    }

    script_data_double_escaped_state {
        // TODO
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

    script_data_double_escaped_dash_dash_state {
        // TODO
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});
