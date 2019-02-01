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

        let test_cases = |chunks: Vec<TextChunk<'_>>, factory: TokenFactory| {
            vec![
                (
                    "Parsed",
                    chunks
                        .iter()
                        .map(|c| c.to_owned())
                        .collect::<Vec<TextChunk<'static>>>(),
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
                (
                    "With prepends and appends",
                    {
                        let mut chunks = chunks
                            .iter()
                            .map(|c| c.to_owned())
                            .collect::<Vec<TextChunk<'static>>>();

                        let first = chunks.first_mut().unwrap();

                        first.prepend("<div>Hey</div>".into());
                        first.prepend(
                            factory
                                .try_start_tag_from("foo", &[], false)
                                .unwrap()
                                .into(),
                        );

                        let last = chunks.last_mut().unwrap();

                        last.append(factory.try_end_tag_from("foo").unwrap().into());
                        last.append("<!-- 42 -->".into());

                        chunks
                    },
                    concat!(
                        "<div>Hey</div><foo>",
                        "Lorem ipsum dolor sit amet, < consectetur adipiscing elit, sed do eiusmod \
                         tempor incididunt ut labore et dolore > magna aliqua. Ut enim ad minim \
                         veniam, quis nostrud exercitation & ullamco laboris < nisi >> ut aliquip \
                         ex ea > commodo > consequat.",
                        "<!-- 42 --></foo>"
                    )
                ),
                (
                    "Removed",
                    {
                        let mut chunks = chunks
                            .iter()
                            .map(|c| c.to_owned())
                            .collect::<Vec<TextChunk<'static>>>();

                        chunks.first_mut().unwrap().prepend("<before>".into());
                        chunks.last_mut().unwrap().append("<after>".into());
                        chunks.iter_mut().for_each(|c| c.remove());

                        chunks
                    },
                    "<before><after>",
                ),
                (
                    "Replaced",
                    {
                        let mut chunk = chunks[0].to_owned();

                        chunk.prepend("<before>".into());
                        chunk.append("<after>".into());

                        chunk.add_replacement("<div></div>".into());
                        chunk.add_replacement(factory.try_comment_from("42").unwrap().into());

                        vec![chunk]
                    },
                    "<before><div></div><!--42--><after>",
                ),
            ]
        };

        serialization_test!(TextChunk, TEXT, src, test_cases);
    });
});
