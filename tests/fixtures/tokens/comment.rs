use cool_thing::token::{Comment, CommentTextError, TokenFactory};
use encoding_rs::{EUC_JP, UTF_8};

test_fixture!("Comment token", {
    test("Comment closing sequence in text", {
        let factory = TokenFactory::new(UTF_8);

        let errs = [
            factory.try_comment_from("foo -- bar --> baz").unwrap_err(),
            factory
                .try_comment_from("")
                .unwrap()
                .set_text("foo -- bar --> baz")
                .unwrap_err(),
        ];

        for err in errs.into_iter() {
            assert_eq!(
                err.downcast_ref::<CommentTextError>().cloned(),
                Some(CommentTextError::CommentClosingSequence)
            );
        }
    });

    test("Encoding-unmappable characters text", {
        let factory = TokenFactory::new(EUC_JP);

        let errs = [
            factory.try_comment_from("foo\u{00F8}bar").unwrap_err(),
            factory
                .try_comment_from("")
                .unwrap()
                .set_text("foo\u{00F8}bar")
                .unwrap_err(),
        ];

        for err in errs.into_iter() {
            assert_eq!(
                err.downcast_ref::<CommentTextError>().cloned(),
                Some(CommentTextError::UnencodableCharacter)
            );
        }
    });

    test("Serialization", {
        let src = "<!-- foo -- bar -->";

        let test_cases = |comments: Vec<Comment<'_>>, _| {
            vec![
                ("Parsed", comments[0].to_owned(), "<!-- foo -- bar -->"),
                (
                    "Modified text",
                    {
                        let mut comment = comments[0].to_owned();

                        comment.set_text("42 <!-").unwrap();

                        comment
                    },
                    "<!--42 <!--->",
                ),
            ]
        };

        serialization_test!(Comment, COMMENTS, src, test_cases);
    });
});
