use crate::harness::ASCII_COMPATIBLE_ENCODINGS;
use cool_thing::{create_element, AttributeNameError, Element, StartTag, TagNameError};
use encoding_rs::{Encoding, EUC_JP, UTF_8};

fn parse_element(
    input: &'static str,
    encoding: &'static Encoding,
    mut handler: impl FnMut(Element<'_, '_>),
) {
    parse_token!(input, encoding, StartTag, |t: &mut StartTag| {
        handler(create_element(t));
    });
}

test_fixture!("Element", {
    test("Empty tag name", {
        parse_element("<div>", UTF_8, |mut el| {
            let err = el.set_tag_name("").unwrap_err();

            assert_eq!(err, TagNameError::Empty);
        });
    });

    test("Forbidden characters in tag name", {
        parse_element("<div>", UTF_8, |mut el| {
            for &ch in &[' ', '\n', '\r', '\t', '\x0C', '/', '>'] {
                let err = el.set_tag_name(&format!("foo{}bar", ch)).unwrap_err();

                assert_eq!(err, TagNameError::ForbiddenCharacter(ch));
            }
        });
    });

    test("Encoding-unmappable characters in tag name", {
        parse_element("<div>", EUC_JP, |mut el| {
            let err = el.set_tag_name("foo\u{00F8}bar").unwrap_err();

            assert_eq!(err, TagNameError::UnencodableCharacter);
        });
    });

    test("Invalid first character of tag name", {
        parse_element("<div>", UTF_8, |mut el| {
            let err = el.set_tag_name("1foo").unwrap_err();

            assert_eq!(err, TagNameError::InvalidFirstCharacter);
        });
    });

    test("Empty attribute name", {
        parse_element("<div>", UTF_8, |mut el| {
            let err = el.set_attribute("", "").unwrap_err();

            assert_eq!(err, AttributeNameError::Empty);
        });
    });

    test("Forbidden characters in attribute name", {
        parse_element("<div>", UTF_8, |mut el| {
            for &ch in &[' ', '\n', '\r', '\t', '\x0C', '/', '>', '='] {
                let err = el.set_attribute(&format!("foo{}bar", ch), "").unwrap_err();

                assert_eq!(err, AttributeNameError::ForbiddenCharacter(ch));
            }
        });
    });

    test("Encoding-unmappable characters in attribute name", {
        parse_element("<div>", EUC_JP, |mut el| {
            let err = el.set_attribute("foo\u{00F8}bar", "").unwrap_err();

            assert_eq!(err, AttributeNameError::UnencodableCharacter);
        });
    });

    test("Tag name getter and setter", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_element("<Foo>", enc, |mut el| {
                assert_eq!(el.tag_name(), "foo", "Encoding: {}", enc.name());

                el.set_tag_name("BaZ").unwrap();

                assert_eq!(el.tag_name(), "baz", "Encoding: {}", enc.name());
            });
        }
    });

    test("Attribute list", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, |el| {
                assert_eq!(el.attributes().len(), 2, "Encoding: {}", enc.name());
                assert_eq!(
                    el.attributes()[0].name(),
                    "foo1",
                    "Encoding: {}",
                    enc.name()
                );
                assert_eq!(
                    el.attributes()[1].name(),
                    "foo2",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(
                    el.attributes()[0].value(),
                    "Bar1",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(
                    el.attributes()[1].value(),
                    "Bar2",
                    "Encoding: {}",
                    enc.name()
                );
            });
        }
    });

    test("Get attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, |el| {
                assert_eq!(
                    el.get_attribute("fOo1").unwrap(),
                    "Bar1",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(
                    el.get_attribute("Foo1").unwrap(),
                    "Bar1",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(
                    el.get_attribute("FOO2").unwrap(),
                    "Bar2",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(
                    el.get_attribute("foo2").unwrap(),
                    "Bar2",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(el.get_attribute("foo3"), None, "Encoding: {}", enc.name());
            });
        }
    });

    test("Has attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, |el| {
                assert!(el.has_attribute("FOo1"), "Encoding: {}", enc.name());
                assert!(el.has_attribute("foo1"), "Encoding: {}", enc.name());
                assert!(el.has_attribute("FOO2"), "Encoding: {}", enc.name());
                assert!(!el.has_attribute("foo3"), "Encoding: {}", enc.name());
            });
        }
    });

    test("Set attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_element("<div>", enc, |mut el| {
                el.set_attribute("Foo", "Bar1").unwrap();

                assert_eq!(
                    el.get_attribute("foo").unwrap(),
                    "Bar1",
                    "Encoding: {}",
                    enc.name()
                );

                el.set_attribute("fOO", "Bar2").unwrap();

                assert_eq!(
                    el.get_attribute("foo").unwrap(),
                    "Bar2",
                    "Encoding: {}",
                    enc.name()
                );
            });
        }
    });

    test("Remove attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            parse_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, |mut el| {
                el.remove_attribute("Unknown");

                assert_eq!(el.attributes().len(), 2, "Encoding: {}", enc.name());

                el.remove_attribute("Foo1");

                assert_eq!(el.attributes().len(), 1, "Encoding: {}", enc.name());
                assert_eq!(el.get_attribute("foo1"), None, "Encoding: {}", enc.name());

                el.remove_attribute("FoO2");

                assert!(el.attributes().is_empty(), "Encoding: {}", enc.name());
                assert_eq!(el.get_attribute("foo2"), None, "Encoding: {}", enc.name());
            });
        }
    });
});
