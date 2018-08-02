define_state_group!(tag_states_group = {

    tag_open_state {
        b'!'  => ( --> markup_declaration_open_state )
        b'/'  => ( --> end_tag_open_state )
        alpha => ( create_start_tag; start_token_part; --> tag_name_state )
        b'?'  => ( start_token_part; --> bogus_comment_state )
        eof   => ( emit_chars; emit_eof; )
        _     => ( emit_chars; reconsume in data_state )
    }

    end_tag_open_state {
        alpha => ( create_end_tag; start_token_part; --> tag_name_state )
        b'>'  => ( --> data_state )
        eof => ( emit_chars; emit_eof; )
        _   => ( start_token_part; --> bogus_comment_state )
    }

    markup_declaration_open_state {
        //TODO
        [ "--" ]                     => ()
        [ "DOCTYPE"; ignore_case ]   => ()
        [ "[CDATA[" ]                => ()
        eof                          => ( emit_eof; )
        _                            => ( emit_eof; )
    }

    tag_name_state {
        whitespace => ( finish_tag_name; --> before_attribute_name_state )
        b'/'       => ( finish_tag_name; --> self_closing_start_tag_state )
        b'>'       => ( finish_tag_name; emit_current_token; --> data_state )
        eof        => ( emit_eof; )
        _          => ()
    }

    self_closing_start_tag_state {
        b'>' => ( mark_as_self_closing; emit_current_token; --> data_state )
        eof  => ( emit_eof; )
        _    => ( reconsume in before_attribute_name_state )
    }
});
