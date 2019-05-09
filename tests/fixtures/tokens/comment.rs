use cool_thing::{Comment, CommentTextError, ContentType, UserData};
use encoding_rs::{EUC_JP, UTF_8};

test_fixture!("Comment token", {
    test("Comment closing sequence in text", {
        parse_token!("<!-- foo -->", UTF_8, Comment, |c: &mut Comment| {
            let err = c.set_text("foo -- bar --> baz").unwrap_err();

            assert_eq!(err, CommentTextError::CommentClosingSequence);
        });
    });

    test("Encoding-unmappable characters text", {
        parse_token!("<!-- foo -->", EUC_JP, Comment, |c: &mut Comment| {
            let err = c.set_text("foo\u{00F8}bar").unwrap_err();

            assert_eq!(err, CommentTextError::UnencodableCharacter);
        });
    });

    test("User data", {
        parse_token!("<!-- foo -->", UTF_8, Comment, |c: &mut Comment| {
            c.set_user_data(42usize);

            assert_eq!(
                *c.user_data().unwrap().downcast_ref::<usize>().unwrap(),
                42usize
            );
        });
    });

    test("Serialization", {
        serialization_test!(
            "<!-- foo -- bar -->",
            Comment,
            &[
                ("Parsed", Box::new(|_, _| {}), "<!-- foo -- bar -->"),
                (
                    "Modified text",
                    Box::new(|c, _| {
                        c.set_text("42 <!-").unwrap();
                    }),
                    "<!--42 <!--->",
                ),
                (
                    "With prepends and appends",
                    Box::new(|c, _| {
                        c.before("<span>", ContentType::Text);
                        c.before("<div>Hey</div>", ContentType::Html);
                        c.before("<foo>", ContentType::Html);
                        c.after("</foo>", ContentType::Html);
                        c.after("<!-- 42 -->", ContentType::Html);
                        c.after("<foo & bar>", ContentType::Text);
                    }),
                    concat!(
                        "&lt;span&gt;<div>Hey</div><foo><!-- foo -- bar -->",
                        "&lt;foo &amp; bar&gt;<!-- 42 --></foo>",
                    )
                ),
                (
                    "Removed",
                    Box::new(|c, _| {
                        assert!(!c.removed());

                        c.remove();

                        assert!(c.removed());

                        c.before("<before>", ContentType::Html);
                        c.after("<after>", ContentType::Html);
                    }),
                    "<before><after>",
                ),
                (
                    "Replaced",
                    Box::new(|c, _| {
                        c.before("<before>", ContentType::Html);
                        c.after("<after>", ContentType::Html);

                        assert!(!c.removed());

                        c.replace("<div></div>", ContentType::Html);
                        c.replace("<!--42-->", ContentType::Html);
                        c.replace("<foo & bar>", ContentType::Text);

                        assert!(c.removed());
                    }),
                    "<before><div></div><!--42-->&lt;foo &amp; bar&gt;<after>",
                ),
            ]
        );
    });
});
