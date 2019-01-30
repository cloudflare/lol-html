use cool_thing::token::{TextChunk, TextError, TokenFactory};
use encoding_rs::{EUC_JP, UTF_8};

test_fixture!("Text chunk token", {
    test("Construction", {
        let chunk = TokenFactory::new(UTF_8).new_text("Hey test 42");

        assert_eq!(chunk.as_str(), "Hey test 42");
    });

    test("Encoding-unmappable characters", {
        let factory = TokenFactory::new(EUC_JP);

        let errs = [
            factory.try_script_text_from("foo\u{00F8}bar").unwrap_err(),
            factory
                .try_stylesheet_text_from("foo\u{00F8}bar")
                .unwrap_err(),
        ];

        for err in &errs {
            assert_eq!(
                err.downcast_ref::<TextError>().cloned(),
                Some(TextError::UnencodableCharacter)
            );
        }
    });

    test("End tag in text", {
        let factory = TokenFactory::new(UTF_8);

        let errs = [
            (
                factory.try_script_text_from("foo </script>").unwrap_err(),
                Some(TextError::ScriptEndTagInScriptText),
            ),
            (
                factory
                    .try_stylesheet_text_from("foo </style>")
                    .unwrap_err(),
                Some(TextError::StyleEndTagInStylesheetText),
            ),
        ];

        for (err, expected_err) in &errs {
            assert_eq!(err.downcast_ref::<TextError>().cloned(), *expected_err);
        }
    });

    test("Serialization", {
        let src =
            "Lorem ipsum dolor sit amet, < consectetur adipiscing elit, sed do eiusmod tempor \
             incididunt ut labore et dolore > magna aliqua. Ut enim ad minim veniam, quis nostrud \
             exercitation & ullamco laboris < nisi >> ut aliquip ex ea > commodo > consequat.";

        let escaped =
            "Lorem ipsum dolor sit amet, &lt; consectetur adipiscing elit, sed do eiusmod \
             tempor incididunt ut labore et dolore &gt; magna aliqua. Ut enim ad minim veniam, \
             quis nostrud exercitation &amp; ullamco laboris &lt; nisi &gt;&gt; ut aliquip ex \
             ea &gt; commodo &gt; consequat.";

        let test_cases = |chunks: Vec<TextChunk<'_>>, enc| {
            let factory = TokenFactory::new(enc);

            vec![
                (
                    "Parsed",
                    chunks
                        .iter()
                        .map(|c| c.to_owned())
                        .collect::<Vec<TextChunk<'_>>>(),
                    src,
                ),
                ("Custom", vec![factory.new_text(src)], escaped),
                (
                    "Custom script",
                    vec![factory.try_script_text_from(src).unwrap()],
                    src,
                ),
                (
                    "Custom stylesheet",
                    vec![factory.try_stylesheet_text_from(src).unwrap()],
                    src,
                ),
            ]
        };

        serialization_test!(TextChunk, TEXT, src, test_cases);
    });
});
