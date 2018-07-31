define_state_group!(data_states_group = {

    pub data_state <-- ( start_raw; ) {
        b'<' => ( emit_chars; start_raw; --> tag_open_state )
        eof  => ( emit_chars; emit_eof; )
        _    => ()
    }

    tag_open_state {
        b'!'  => ( --> markup_declaration_open_state )
        b'/'  => ( --> end_tag_open_state )
        alpha => ( create_start_tag; start_token_part; --> tag_name_state )
        b'?'  => ( start_token_part; --> bogus_comment_state )
        eof   => ( emit_chars; emit_eof; )
        _     => ( emit_chars; reconsume in data_state )
    }

    markup_declaration_open_state {
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

    bogus_comment_state {
        b'>' => ( emit_comment; --> data_state )
        eof  => ( emit_comment; emit_eof; )
        _    => ()
    }

    tag_name_state {
        whitespace => ( --> before_attribute_name_state )
        b'/'       => ( --> self_closing_start_tag_state )
        b'>'       => ( finish_tag_name; emit_current_token; --> data_state )
        eof        => ( emit_eof; )
        _          => ()
    }

    before_attribute_name_state {
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

    self_closing_start_tag_state {
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});
