macro_rules! serialization_test {
    ($input:expr, $TokenType:ident, $test_cases:expr) => {
        use crate::harness::ASCII_COMPATIBLE_ENCODINGS;
        use cool_thing::Serialize;
        use encoding_rs::Encoding;

        // NOTE: give test cases type annotation to avoid boilerplate code in tests.
        let test_cases: &[(
            &'static str,
            Box<Fn(&mut $TokenType<'_>, &'static Encoding)>,
            &'static str,
        )] = $test_cases;

        for encoding in ASCII_COMPATIBLE_ENCODINGS.iter() {
            for (case_name, transform, expected) in test_cases {
                parse_token!($input, encoding, $TokenType, |t: &mut $TokenType<'_>| {
                    let mut bytes = Vec::new();

                    transform(t, encoding);

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
