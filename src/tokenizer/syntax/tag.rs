define_state_group!(tag_states_group = {

     tag_name_state {
        whitespace => ( finish_tag_name; --> before_attribute_name_state )
        b'/'       => ( finish_tag_name; --> self_closing_start_tag_state )
        b'>'       => ( finish_tag_name; emit_current_token; --> data_state )
        eof        => ( emit_eof; )
        _          => ()
    }

    before_attribute_name_state {
        whitespace => ()
        b'/'       => ( --> self_closing_start_tag_state )
        b'>'       => ( emit_current_token; --> data_state )
        eof        => ( emit_eof; )
        _          => ( start_attr; --> attribute_name_state )
    }

    attribute_name_state {
        whitespace => ( finish_attr_name; --> after_attribute_name_state )
        b'/'       => ( finish_attr_name; finish_attr; --> self_closing_start_tag_state )
        b'>'       => ( finish_attr_name; finish_attr; emit_current_token; --> data_state )
        b'='       => ( finish_attr_name; --> before_attribute_value_state )
        eof        => ( emit_eof; )
        _          => ()
    }

    after_attribute_name_state {
        whitespace => ()
        b'/'       => ( finish_attr; --> self_closing_start_tag_state )
        b'='       => ( --> before_attribute_value_state )
        b'>'       => ( finish_attr; emit_current_token; --> data_state )
        eof        => ( emit_eof; )
        _          => ( finish_attr; start_attr; --> attribute_name_state )
    }

    before_attribute_value_state {
        whitespace => ()
        b'"'       => ( set_closing_quote_to_double; --> attribute_value_quoted_state )
        b'\''      => ( set_closing_quote_to_single; --> attribute_value_quoted_state )
        b'>'       => ( finish_attr; emit_current_token; --> data_state )
        eof        => ( emit_eof; )
        _          => ( reconsume in attribute_value_unquoted_state )
    }

    attribute_value_quoted_state <-- ( start_token_part; ) {
        closing_quote => ( finish_attr_value; finish_attr; --> after_attribute_value_quoted_state )
        eof           => ( emit_eof; )
        _             => ()
    }

    after_attribute_value_quoted_state {
        whitespace => ( --> before_attribute_name_state )
        b'/'       => ( --> self_closing_start_tag_state )
        b'>'       => ( emit_current_token; --> data_state )
        eof        => ( emit_eof; )
        _          => ( reconsume in before_attribute_name_state )
    }

    attribute_value_unquoted_state <-- ( start_token_part; ) {
        whitespace => ( finish_attr_value; finish_attr; --> before_attribute_name_state )
        b'>'       => ( finish_attr_value; finish_attr; emit_current_token; --> data_state )
        eof        => ( emit_eof; )
        _          => ()
    }

    self_closing_start_tag_state {
        b'>' => ( mark_as_self_closing; emit_current_token; --> data_state )
        eof  => ( emit_eof; )
        _    => ( reconsume in before_attribute_name_state )
    }

    end_tag_open_state {
        // TODO
        eof => ( emit_eof; )
        _   => ( emit_eof; )
    }

});
