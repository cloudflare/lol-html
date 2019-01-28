use crate::harness::ASCII_COMPATIBLE_ENCODINGS;
use cool_thing::token::{AttributeNameError, StartTag, TagNameError, TokenFactory};
use encoding_rs::{EUC_JP, UTF_8};

test_fixture!("Start tag token", {
    test("Empty tag name", {
        let factory = TokenFactory::new(UTF_8);

        let errs = [
            factory.try_start_tag_from("", &[], false).unwrap_err(),
            factory
                .try_start_tag_from("foo", &[], false)
                .unwrap()
                .set_name("")
                .unwrap_err(),
        ];

        for err in errs.into_iter() {
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
                factory.try_start_tag_from(name, &[], false).unwrap_err(),
                factory
                    .try_start_tag_from("foo", &[], false)
                    .unwrap()
                    .set_name(name)
                    .unwrap_err(),
            ];

            for err in errs.into_iter() {
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
            factory
                .try_start_tag_from("foo\u{00F8}bar", &[], false)
                .unwrap_err(),
            factory
                .try_start_tag_from("foo", &[], false)
                .unwrap()
                .set_name("foo\u{00F8}bar")
                .unwrap_err(),
        ];

        for err in errs.into_iter() {
            assert_eq!(
                err.downcast_ref::<TagNameError>().cloned(),
                Some(TagNameError::UnencodableCharacter)
            );
        }
    });

    test("Invalid first character of tag name", {
        let factory = TokenFactory::new(EUC_JP);

        let errs = [
            factory.try_start_tag_from("1foo", &[], false).unwrap_err(),
            factory
                .try_start_tag_from("foo", &[], false)
                .unwrap()
                .set_name("-bar")
                .unwrap_err(),
        ];

        for err in errs.into_iter() {
            assert_eq!(
                err.downcast_ref::<TagNameError>().cloned(),
                Some(TagNameError::InvalidFirstCharacter)
            );
        }
    });

    test("Empty attribute name", {
        let factory = TokenFactory::new(UTF_8);

        let errs = [
            factory
                .try_start_tag_from("foo", &[("", "")], false)
                .unwrap_err(),
            factory
                .try_start_tag_from("foo", &[], false)
                .unwrap()
                .set_attribute("", "")
                .unwrap_err(),
        ];

        for err in errs.into_iter() {
            assert_eq!(
                err.downcast_ref::<AttributeNameError>().cloned(),
                Some(AttributeNameError::Empty)
            );
        }
    });

    test("Forbidden characters in attribute name", {
        let factory = TokenFactory::new(UTF_8);

        for &ch in &[' ', '\n', '\r', '\t', '\x0C', '/', '>', '='] {
            let name = &format!("foo{}bar", ch);

            let errs = [
                factory
                    .try_start_tag_from("foo", &[(name, "")], false)
                    .unwrap_err(),
                factory
                    .try_start_tag_from("foo", &[], false)
                    .unwrap()
                    .set_attribute(name, "")
                    .unwrap_err(),
            ];

            for err in errs.into_iter() {
                assert_eq!(
                    err.downcast_ref::<AttributeNameError>().cloned(),
                    Some(AttributeNameError::ForbiddenCharacter(ch))
                );
            }
        }
    });

    test("Encoding-unmappable characters in name", {
        let factory = TokenFactory::new(EUC_JP);

        let errs = [
            factory
                .try_start_tag_from("foo", &[("foo\u{00F8}bar", "")], false)
                .unwrap_err(),
            factory
                .try_start_tag_from("foo", &[], false)
                .unwrap()
                .set_attribute("foo\u{00F8}bar", "")
                .unwrap_err(),
        ];

        for err in errs.into_iter() {
            assert_eq!(
                err.downcast_ref::<AttributeNameError>().cloned(),
                Some(AttributeNameError::UnencodableCharacter)
            );
        }
    });

    test("Name getter and setter", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let mut tag = TokenFactory::new(enc)
                .try_start_tag_from("Foo", &[], false)
                .unwrap();

            assert_eq!(tag.name(), "foo", "Encoding: {}", enc.name());

            tag.set_name("BaZ").unwrap();

            assert_eq!(tag.name(), "baz", "Encoding: {}", enc.name());
        }
    });

    test("Attribute list", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let tag = TokenFactory::new(enc)
                .try_start_tag_from("Foo", &[("Foo1", "Bar1"), ("Foo2", "Bar2")], false)
                .unwrap();

            assert_eq!(tag.attributes().len(), 2, "Encoding: {}", enc.name());

            assert_eq!(
                tag.attributes()[0].name(),
                "foo1",
                "Encoding: {}",
                enc.name()
            );

            assert_eq!(
                tag.attributes()[1].name(),
                "foo2",
                "Encoding: {}",
                enc.name()
            );

            assert_eq!(
                tag.attributes()[0].value(),
                "Bar1",
                "Encoding: {}",
                enc.name()
            );

            assert_eq!(
                tag.attributes()[1].value(),
                "Bar2",
                "Encoding: {}",
                enc.name()
            );
        }
    });

    test("Get attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let tag = TokenFactory::new(enc)
                .try_start_tag_from("Foo", &[("Foo1", "Bar1"), ("Foo2", "Bar2")], false)
                .unwrap();

            assert_eq!(
                tag.get_attribute("fOo1"),
                Some("Bar1".into()),
                "Encoding: {}",
                enc.name()
            );

            assert_eq!(
                tag.get_attribute("Foo1"),
                Some("Bar1".into()),
                "Encoding: {}",
                enc.name()
            );

            assert_eq!(
                tag.get_attribute("FOO2"),
                Some("Bar2".into()),
                "Encoding: {}",
                enc.name()
            );

            assert_eq!(
                tag.get_attribute("foo2"),
                Some("Bar2".into()),
                "Encoding: {}",
                enc.name()
            );

            assert_eq!(tag.get_attribute("foo3"), None, "Encoding: {}", enc.name());
        }
    });

    test("Has attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let tag = TokenFactory::new(enc)
                .try_start_tag_from("Foo", &[("Foo1", "Bar1"), ("Foo2", "Bar2")], false)
                .unwrap();

            assert!(tag.has_attribute("FOo1"), "Encoding: {}", enc.name());
            assert!(tag.has_attribute("foo1"), "Encoding: {}", enc.name());
            assert!(tag.has_attribute("FOO2"), "Encoding: {}", enc.name());
            assert!(!tag.has_attribute("foo3"), "Encoding: {}", enc.name());
        }
    });

    test("Set attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let mut tag = TokenFactory::new(enc)
                .try_start_tag_from("Foo", &[], false)
                .unwrap();

            tag.set_attribute("Foo", "Bar1").unwrap();

            assert_eq!(
                tag.get_attribute("foo"),
                Some("Bar1".into()),
                "Encoding: {}",
                enc.name()
            );

            tag.set_attribute("fOO", "Bar2").unwrap();

            assert_eq!(
                tag.get_attribute("foo"),
                Some("Bar2".into()),
                "Encoding: {}",
                enc.name()
            );
        }
    });

    test("Remove attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let mut tag = TokenFactory::new(enc)
                .try_start_tag_from("Foo", &[("Foo1", "Bar1"), ("Foo2", "Bar2")], false)
                .unwrap();

            tag.remove_attribute("Unknown");

            assert_eq!(tag.attributes().len(), 2, "Encoding: {}", enc.name());

            tag.remove_attribute("Foo1");

            assert_eq!(tag.attributes().len(), 1, "Encoding: {}", enc.name());
            assert_eq!(tag.get_attribute("foo1"), None, "Encoding: {}", enc.name());

            tag.remove_attribute("FoO2");

            assert!(tag.attributes().is_empty(), "Encoding: {}", enc.name());
            assert_eq!(tag.get_attribute("foo2"), None, "Encoding: {}", enc.name());
        }
    });

    test("Self closing flag", {
        let mut tag = TokenFactory::new(UTF_8)
            .try_start_tag_from("Foo", &[], true)
            .unwrap();

        assert!(tag.self_closing());

        tag.set_self_closing(false);

        assert!(!tag.self_closing());
    });

    test("Serialization", {
        let src = r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#;

        let test_cases = |tags: Vec<StartTag<'_>>, _| {
            vec![
                (
                    "Parsed",
                    tags[0].to_owned(),
                    r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
                ),
                (
                    "Modified name",
                    {
                        let mut tag = tags[0].to_owned();

                        tag.set_name("div").unwrap();

                        tag
                    },
                    r#"<div a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4>"#,
                ),
                (
                    "Modified single quotted attribute value",
                    {
                        let mut tag = tags[0].to_owned();
                        let new_value = tag.get_attribute("a1").unwrap() + "42";

                        tag.set_attribute("a1", &new_value).unwrap();

                        tag
                    },
                    r#"<a a1="foo &quot; bar &quot; baz42" a2="foo ' bar ' baz" a3=foo/bar a4>"#,
                ),
                (
                    "Modified double quotted attribute value",
                    {
                        let mut tag = tags[0].to_owned();
                        let new_value = tag.get_attribute("a2").unwrap() + "42";

                        tag.set_attribute("a2", &new_value).unwrap();

                        tag
                    },
                    r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz42" a3=foo/bar a4>"#,
                ),
                (
                    "Modified unquotted attribute value",
                    {
                        let mut tag = tags[0].to_owned();
                        let new_value = tag.get_attribute("a3").unwrap() + "42";

                        tag.set_attribute("a3", &new_value).unwrap();

                        tag
                    },
                    r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3="foo/bar42" a4>"#,
                ),
                (
                    "Set value for an attribute without a value",
                    {
                        let mut tag = tags[0].to_owned();

                        tag.set_attribute("a4", "42").unwrap();

                        tag
                    },
                    r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4="42">"#,
                ),
                (
                    "Add attribute",
                    {
                        let mut tag = tags[0].to_owned();

                        tag.set_attribute("a5", r#"42'"42"#).unwrap();

                        tag
                    },
                    r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4 a5="42'&quot;42">"#,
                ),
                (
                    "With self-closing flag",
                    {
                        let mut tag = tags[0].to_owned();

                        tag.set_self_closing(true);

                        tag
                    },
                    r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4 />"#,
                ),
                (
                    "Remove non-existent attribute",
                    {
                        let mut tag = tags[0].to_owned();

                        tag.remove_attribute("a5");

                        tag
                    },
                    r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
                ),
                (
                    "Without attributes",
                    {
                        let mut tag = tags[0].to_owned();

                        for name in &["a1", "a2", "a3", "a4"] {
                            tag.remove_attribute(name);
                        }

                        tag
                    },
                    r#"<a>"#,
                ),
                (
                    "Without attributes self-closing",
                    {
                        let mut tag = tags[0].to_owned();

                        tag.set_self_closing(true);

                        for name in &["a1", "a2", "a3", "a4"] {
                            tag.remove_attribute(name);
                        }

                        tag
                    },
                    r#"<a/>"#,
                ),
            ]
        };

        serialization_test!(StartTag, START_TAGS, src, test_cases);
    });
});
