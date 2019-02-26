use cool_thing::content::{EndTag, TagNameError};
use encoding_rs::{EUC_JP, UTF_8};

test_fixture!("End tag token", {
    test("Empty tag name", {
        parse_token!("</div>", UTF_8, EndTag, |t: &mut EndTag<'_>| {
            let err = t
                .set_name("")
                .unwrap_err()
                .downcast_ref::<TagNameError>()
                .cloned()
                .unwrap();

            assert_eq!(err, TagNameError::Empty);
        });
    });

    test("Forbidden characters in tag name", {
        parse_token!("</div>", UTF_8, EndTag, |t: &mut EndTag<'_>| for &ch in
            &[' ', '\n', '\r', '\t', '\x0C', '/', '>']
        {
            let err = t
                .set_name(&format!("foo{}bar", ch))
                .unwrap_err()
                .downcast_ref::<TagNameError>()
                .cloned()
                .unwrap();

            assert_eq!(err, TagNameError::ForbiddenCharacter(ch));
        });
    });

    test("Encoding-unmappable characters in tag name", {
        parse_token!("</div>", EUC_JP, EndTag, |t: &mut EndTag<'_>| {
            let err = t
                .set_name("foo\u{00F8}bar")
                .unwrap_err()
                .downcast_ref::<TagNameError>()
                .cloned()
                .unwrap();

            assert_eq!(err, TagNameError::UnencodableCharacter);
        });
    });

    test("Invalid first character of tag name", {
        parse_token!("</div>", UTF_8, EndTag, |t: &mut EndTag<'_>| {
            let err = t
                .set_name("1foo")
                .unwrap_err()
                .downcast_ref::<TagNameError>()
                .cloned()
                .unwrap();

            assert_eq!(err, TagNameError::InvalidFirstCharacter);
        });
    });

    test("Serialization", {
        serialization_test!(
            "</div foo=bar>",
            EndTag,
            &[
                ("Parsed", Box::new(|_| {}), "</div foo=bar>"),
                (
                    "Modified name",
                    Box::new(|t| {
                        t.set_name("span").unwrap();
                    }),
                    "</span>",
                ),
                (
                    "With prepends and appends",
                    Box::new(|t| {
                        t.before("<div>Hey</div>");
                        t.before("<foo>");
                        t.after("</foo>");
                        t.after("<!-- 42 -->");
                    }),
                    "<div>Hey</div><foo></div foo=bar><!-- 42 --></foo>",
                ),
                (
                    "Removed",
                    Box::new(|t| {
                        t.remove();
                        t.before("<before>");
                        t.after("<after>");
                    }),
                    "<before><after>",
                ),
                (
                    "Replaced",
                    Box::new(|t| {
                        t.before("<before>");
                        t.after("<after>");
                        t.replace("<div></div>");
                        t.replace("<!--42-->");
                    }),
                    "<before><div></div><!--42--><after>",
                ),
            ]
        );
    });
});
