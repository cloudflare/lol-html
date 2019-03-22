macro_rules! serialization_test {
    ($input:expr, $TokenType:ident, $test_cases:expr) => {
        use crate::harness::{ASCII_COMPATIBLE_ENCODINGS, TestOutput};
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
                    let mut output = TestOutput::new(encoding);

                    transform(t, encoding);

                    t.to_bytes(&mut |c| output.push(c));

                    // NOTE: add finalizing chunk to the output.
                    output.push(&[]);

                    let actual: String = output.into();

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
