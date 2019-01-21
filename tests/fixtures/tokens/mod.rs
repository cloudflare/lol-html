macro_rules! serialization_test {
    ($TokenType:ident, $CAPTURE_FLAG:ident, $input:expr, $get_test_cases:expr) => {
        use crate::harness::parsing::{parse, ChunkedInput};
        use crate::harness::ASCII_COMPATIBLE_ENCODINGS;
        use cool_thing::parser::TextType;
        use cool_thing::token::{Token, TokenCaptureFlags};
        use cool_thing::transform_stream::Serialize;
        use encoding_rs::Encoding;

        fn get_token(enc: &'static Encoding) -> $TokenType<'_> {
            let mut input: ChunkedInput = String::from($input).into();

            let mut token = None;

            input.init(enc).unwrap();

            parse(
                &input,
                TokenCaptureFlags::$CAPTURE_FLAG,
                TextType::Data,
                None,
                Box::new(|t| match t {
                    Token::$TokenType(t) => token = Some(t.to_owned()),
                    _ => unreachable!(),
                }),
            )
            .unwrap();

            token.unwrap()
        }

        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let token = get_token(enc);
            let get_test_cases = $get_test_cases;

            for (case_name, tag, expected) in get_test_cases(token).into_iter() {
                let mut bytes = Vec::new();

                tag.into_bytes(&mut |c| bytes.extend_from_slice(&c));

                let actual = enc.decode(&bytes).0.into_owned();

                assert_eq!(
                    actual,
                    expected,
                    "Test case: {} Encoding: {}",
                    case_name,
                    enc.name()
                );
            }
        }
    };
}

test_modules!(start_tag, end_tag);
