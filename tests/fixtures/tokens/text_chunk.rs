use cool_thing::token::{TextChunk, TokenFactory};
use encoding_rs::UTF_8;

test_fixture!("Text chunk token", {
    test("Construction", {
        let chunk = TokenFactory::new(UTF_8).new_text_chunk("Hey test 42");

        assert_eq!(chunk.as_str(), "Hey test 42");
    });

    test("Serialization", {
        let src =
            "Lorem ipsum dolor sit amet, < consectetur adipiscing elit, sed do eiusmod tempor \
             incididunt ut labore et dolore > magna aliqua. Ut enim ad minim veniam, quis nostrud \
             exercitation ullamco laboris < nisi >> ut aliquip ex ea > commodo > consequat.";

        let test_cases = |chunks: Vec<TextChunk<'_>>| {
            vec![(
                "Parsed",
                chunks
                    .into_iter()
                    .map(|c| c.to_owned())
                    .collect::<Vec<TextChunk<'_>>>(),
                "Lorem ipsum dolor sit amet, &lt; consectetur adipiscing elit, sed do eiusmod \
                 tempor incididunt ut labore et dolore &gt; magna aliqua. Ut enim ad minim veniam, \
                 quis nostrud exercitation ullamco laboris &lt; nisi &gt;&gt; ut aliquip ex ea \
                 &gt; commodo &gt; consequat.",
            )]
        };

        serialization_test!(TextChunk, TEXT, src, test_cases);
    });
});
