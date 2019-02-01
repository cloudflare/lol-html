use cool_thing::token::{EndTag, TagNameError, TokenFactory};
use encoding_rs::{EUC_JP, UTF_8};

test_fixture!("End tag token", {
    test("Empty tag name", {
        let factory = TokenFactory::new(UTF_8);

        let errs = [
            factory.try_end_tag_from("").unwrap_err(),
            factory
                .try_end_tag_from("foo")
                .unwrap()
                .set_name("")
                .unwrap_err(),
        ];

        for err in &errs {
            assert_eq!(
                err.downcast_ref::<TagNameError>().cloned(),
                Some(TagNameError::Empty)
            );
        }
    });

    test("Forbidden characters in tag name", {
        let factory = TokenFactory::new(UTF_8);

        for &ch in &[' ', '\n', '\r', '\t', '\x0C', '/', '>'] {
            let name = &format!("foo{}bar", ch);

            let errs = [
                factory.try_end_tag_from(name).unwrap_err(),
                factory
                    .try_end_tag_from("foo")
                    .unwrap()
                    .set_name(name)
                    .unwrap_err(),
            ];

            for err in &errs {
                assert_eq!(
                    err.downcast_ref::<TagNameError>().cloned(),
                    Some(TagNameError::ForbiddenCharacter(ch))
                );
            }
        }
    });

    test("Encoding-unmappable characters in tag name", {
        let factory = TokenFactory::new(EUC_JP);

        let errs = [
            factory.try_end_tag_from("foo\u{00F8}bar").unwrap_err(),
            factory
                .try_end_tag_from("foo")
                .unwrap()
                .set_name("foo\u{00F8}bar")
                .unwrap_err(),
        ];

        for err in &errs {
            assert_eq!(
                err.downcast_ref::<TagNameError>().cloned(),
                Some(TagNameError::UnencodableCharacter)
            );
        }
    });

    test("Invalid first character of tag name", {
        let factory = TokenFactory::new(EUC_JP);

        let errs = [
            factory.try_end_tag_from("1foo").unwrap_err(),
            factory
                .try_end_tag_from("foo")
                .unwrap()
                .set_name("-bar")
                .unwrap_err(),
        ];

        for err in &errs {
            assert_eq!(
                err.downcast_ref::<TagNameError>().cloned(),
                Some(TagNameError::InvalidFirstCharacter)
            );
        }
    });

    test("Serialization", {
        let src = "</div foo=bar>";

        let test_cases = |tags: Vec<EndTag<'_>>, factory: TokenFactory| {
            vec![
                ("Parsed", tags[0].to_owned(), "</div foo=bar>"),
                (
                    "Modified name",
                    {
                        let mut tag = tags[0].to_owned();

                        tag.set_name("span").unwrap();

                        tag
                    },
                    "</span>",
                ),
                (
                "With prepends and appends",
                {
                        let mut tag = tags[0].to_owned();

                        tag.prepend("<div>Hey</div>".into());
                        tag.prepend(
                            factory
                                .try_start_tag_from("foo", &[], false)
                                .unwrap()
                                .into(),
                        );
                        tag.append(factory.try_end_tag_from("foo").unwrap().into());
                        tag.append("<!-- 42 -->".into());

                        tag
                    },
                    "<div>Hey</div><foo></div foo=bar><!-- 42 --></foo>",
                ),
                (
                    "Removed",
                    {
                        let mut tag = tags[0].to_owned();

                        tag.remove();
                        tag.prepend("<before>".into());
                        tag.append("<after>".into());

                        tag
                    },
                    "<before><after>",
                ),
                (
                    "Replaced",
                    {
                        let mut tag = tags[0].to_owned();

                        tag.prepend("<before>".into());
                        tag.append("<after>".into());

                        tag.add_replacement("<div></div>".into());
                        tag.add_replacement(factory.try_comment_from("42").unwrap().into());

                        tag
                    },
                    "<before><div></div><!--42--><after>",
                ),
            ]
        };

        serialization_test!(EndTag, END_TAGS, src, test_cases);
    });
});
