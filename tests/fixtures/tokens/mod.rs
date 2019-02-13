macro_rules! parse_token {
    ($input:expr, $encoding:expr, $TokenType:ident, $callback:expr) => {{
        use crate::harness::parsing::{parse, ChunkedInput};
        use cool_thing::parser::TextType;
        use cool_thing::token::{Token, TokenCaptureFlags};

        let mut input: ChunkedInput = String::from($input).into();
        let mut emitted = false;

        input.init($encoding, true).unwrap();

        parse(
            &input,
            TokenCaptureFlags::all(),
            TextType::Data,
            None,
            Box::new(|t| match t {
                Token::$TokenType(t) => {
                    // NOTE: we always have two text chunks:
                    // one with the actual text and the second is emitted
                    // on EOF to signify the end of the text node.
                    // We need to invoke callback only for the first one.
                    if !emitted {
                        $callback(t);
                        emitted = true;
                    }
                }
                _ => unreachable!("Input should contain only tokens of the requested type"),
            }),
        )
        .unwrap();
    }};
}

macro_rules! serialization_test {
    ($input:expr, $TokenType:ident, $test_cases:expr) => {
        use crate::harness::ASCII_COMPATIBLE_ENCODINGS;
        use cool_thing::token::Serialize;

        // NOTE: give test cases type annotation to avoid boilerplate code in tests.
        let test_cases: &[(&'static str, Box<Fn(&mut $TokenType<'_>)>, &'static str)] = $test_cases;

        for encoding in ASCII_COMPATIBLE_ENCODINGS.iter() {
            for (case_name, transform, expected) in test_cases {
                parse_token!($input, encoding, $TokenType, |t: &mut $TokenType<'_>| {
                    let mut bytes = Vec::new();

                    transform(t);

                    t.to_bytes(&mut |c| bytes.extend_from_slice(c));

                    let actual = encoding.decode(&bytes).0.into_owned();

                    assert_eq!(
                        actual,
                        *expected,
                        "Test case: {} Encoding: {}",
                        case_name,
                        encoding.name()
                    );
                });
            }
        }
    };
}

test_modules!(start_tag, end_tag, comment, doctype, text_chunk);
