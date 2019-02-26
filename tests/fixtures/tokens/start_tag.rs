use crate::harness::ASCII_COMPATIBLE_ENCODINGS;
use cool_thing::content::{AttributeNameError, StartTag, TagNameError};
use encoding_rs::{EUC_JP, UTF_8};

test_fixture!("Start tag token", {
    test("Empty tag name", {
        parse_token!("<div>", UTF_8, StartTag, |t: &mut StartTag<'_>| {
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
        parse_token!("<div>", UTF_8, StartTag, |t: &mut StartTag<'_>| for &ch in
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
        parse_token!("<div>", EUC_JP, StartTag, |t: &mut StartTag<'_>| {
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
        parse_token!("<div>", UTF_8, StartTag, |t: &mut StartTag<'_>| {
            let err = t
                .set_name("1foo")
                .unwrap_err()
                .downcast_ref::<TagNameError>()
                .cloned()
                .unwrap();

            assert_eq!(err, TagNameError::InvalidFirstCharacter);
        });
    });

    test("Empty attribute name", {
        parse_token!("<div>", UTF_8, StartTag, |t: &mut StartTag<'_>| {
            let err = t
                .set_attribute("", "")
                .unwrap_err()
                .downcast_ref::<AttributeNameError>()
                .cloned()
                .unwrap();

            assert_eq!(err, AttributeNameError::Empty);
        });
    });

    test("Forbidden characters in attribute name", {
        parse_token!("<div>", UTF_8, StartTag, |t: &mut StartTag<'_>| for &ch in
            &[' ', '\n', '\r', '\t', '\x0C', '/', '>', '=']
        {
            let err = t
                .set_attribute(&format!("foo{}bar", ch), "")
                .unwrap_err()
                .downcast_ref::<AttributeNameError>()
                .cloned()
                .unwrap();

            assert_eq!(err, AttributeNameError::ForbiddenCharacter(ch));
        });
    });

    test("Encoding-unmappable characters in attribute name", {
        parse_token!("<div>", EUC_JP, StartTag, |t: &mut StartTag<'_>| {
            let err = t
                .set_attribute("foo\u{00F8}bar", "")
                .unwrap_err()
                .downcast_ref::<AttributeNameError>()
                .cloned()
                .unwrap();

            assert_eq!(err, AttributeNameError::UnencodableCharacter);
        });
    });

    test("Name getter and setter", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_token!("<Foo>", enc, StartTag, |t: &mut StartTag<'_>| {
                assert_eq!(t.name(), "foo", "Encoding: {}", enc.name());

                t.set_name("BaZ").unwrap();

                assert_eq!(t.name(), "baz", "Encoding: {}", enc.name());
            });
        }
    });

    test("Attribute list", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_token!(
                "<Foo Foo1=Bar1 Foo2=Bar2>",
                enc,
                StartTag,
                |t: &mut StartTag<'_>| {
                    assert_eq!(t.attributes().len(), 2, "Encoding: {}", enc.name());
                    assert_eq!(t.attributes()[0].name(), "foo1", "Encoding: {}", enc.name());
                    assert_eq!(t.attributes()[1].name(), "foo2", "Encoding: {}", enc.name());

                    assert_eq!(
                        t.attributes()[0].value(),
                        "Bar1",
                        "Encoding: {}",
                        enc.name()
                    );

                    assert_eq!(
                        t.attributes()[1].value(),
                        "Bar2",
                        "Encoding: {}",
                        enc.name()
                    );
                }
            );
        }
    });

    test("Get attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_token!(
                "<Foo Foo1=Bar1 Foo2=Bar2>",
                enc,
                StartTag,
                |t: &mut StartTag<'_>| {
                    assert_eq!(
                        t.get_attribute("fOo1").unwrap(),
                        "Bar1",
                        "Encoding: {}",
                        enc.name()
                    );

                    assert_eq!(
                        t.get_attribute("Foo1").unwrap(),
                        "Bar1",
                        "Encoding: {}",
                        enc.name()
                    );

                    assert_eq!(
                        t.get_attribute("FOO2").unwrap(),
                        "Bar2",
                        "Encoding: {}",
                        enc.name()
                    );

                    assert_eq!(
                        t.get_attribute("foo2").unwrap(),
                        "Bar2",
                        "Encoding: {}",
                        enc.name()
                    );

                    assert_eq!(t.get_attribute("foo3"), None, "Encoding: {}", enc.name());
                }
            );
        }
    });
    test("Has attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_token!(
                "<Foo Foo1=Bar1 Foo2=Bar2>",
                enc,
                StartTag,
                |t: &mut StartTag<'_>| {
                    assert!(t.has_attribute("FOo1"), "Encoding: {}", enc.name());
                    assert!(t.has_attribute("foo1"), "Encoding: {}", enc.name());
                    assert!(t.has_attribute("FOO2"), "Encoding: {}", enc.name());
                    assert!(!t.has_attribute("foo3"), "Encoding: {}", enc.name());
                }
            );
        }
    });

    test("Set attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_token!("<div>", enc, StartTag, |t: &mut StartTag<'_>| {
                t.set_attribute("Foo", "Bar1").unwrap();

                assert_eq!(
                    t.get_attribute("foo").unwrap(),
                    "Bar1",
                    "Encoding: {}",
                    enc.name()
                );

                t.set_attribute("fOO", "Bar2").unwrap();

                assert_eq!(
                    t.get_attribute("foo").unwrap(),
                    "Bar2",
                    "Encoding: {}",
                    enc.name()
                );
            });
        }
    });

    test("Remove attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_token!(
                "<Foo Foo1=Bar1 Foo2=Bar2>",
                enc,
                StartTag,
                |t: &mut StartTag<'_>| {
                    t.remove_attribute("Unknown");

                    assert_eq!(t.attributes().len(), 2, "Encoding: {}", enc.name());

                    t.remove_attribute("Foo1");

                    assert_eq!(t.attributes().len(), 1, "Encoding: {}", enc.name());
                    assert_eq!(t.get_attribute("foo1"), None, "Encoding: {}", enc.name());

                    t.remove_attribute("FoO2");

                    assert!(t.attributes().is_empty(), "Encoding: {}", enc.name());
                    assert_eq!(t.get_attribute("foo2"), None, "Encoding: {}", enc.name());
                }
            );
        }
    });

    test("Self closing flag", {
        parse_token!("<div/>", UTF_8, StartTag, |t: &mut StartTag<'_>| {
            assert!(t.self_closing());

            t.set_self_closing(false);

            assert!(!t.self_closing());
        });
    });

    test("Serialization", {
        serialization_test!(
            r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
            StartTag,
            &[
            (
                "Parsed",
                Box::new(|_| {}),
                r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
            ),
            (
                "Modified name",
                Box::new(|t| {
                    t.set_name("div").unwrap();
                }),
                r#"<div a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4>"#,
            ),
            (
                "Modified single quotted attribute value",
                Box::new(|t| {
                    let new_value = t.get_attribute("a1").unwrap() + "42";

                    t.set_attribute("a1", &new_value).unwrap();
                }),
                r#"<a a1="foo &quot; bar &quot; baz42" a2="foo ' bar ' baz" a3=foo/bar a4>"#,
            ),
            (
                "Modified double quotted attribute value",
                Box::new(|t| {
                    let new_value = t.get_attribute("a2").unwrap() + "42";

                    t.set_attribute("a2", &new_value).unwrap();
                }),
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz42" a3=foo/bar a4>"#,
            ),
            (
                "Modified unquotted attribute value",
                Box::new(|t| {
                    let new_value = t.get_attribute("a3").unwrap() + "42";

                    t.set_attribute("a3", &new_value).unwrap();
                }),
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3="foo/bar42" a4>"#,
            ),
            (
                "Set value for an attribute without a value",
                Box::new(|t| {
                    t.set_attribute("a4", "42").unwrap();
                }),
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4="42">"#,
            ),
            (
                "Add attribute",
                Box::new(|t| {
                    t.set_attribute("a5", r#"42'"42"#).unwrap();
                }),
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4 a5="42'&quot;42">"#,
            ),
            (
                "With self-closing flag",
                Box::new(|t| {
                    t.set_self_closing(true);
                }),
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4 />"#,
            ),
            (
                "Remove non-existent attribute",
                Box::new(|t| t.remove_attribute("a5")),
                r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
            ),
            (
                "Without attributes",
                Box::new(|t| {
                    for name in &["a1", "a2", "a3", "a4"] {
                        t.remove_attribute(name);
                    }
                }),
                r#"<a>"#,
            ),
            (
                "Without attributes self-closing",
                Box::new(|t| {
                    t.set_self_closing(true);

                    for name in &["a1", "a2", "a3", "a4"] {
                        t.remove_attribute(name);
                    }
                }),
                r#"<a/>"#,
            ),
            (
                "With prepends and appends",
                Box::new(|t| {
                    t.before("<div>Hey</div>");
                    t.before("<foo>");
                    t.after("</foo>");
                    t.after("<!-- 42 -->");
                }),
                concat!(
                    "<div>Hey</div><foo>",
                    r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
                    "<!-- 42 --></foo>"
                ),
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
        ]);
    });
});
