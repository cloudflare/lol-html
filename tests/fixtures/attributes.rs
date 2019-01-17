use crate::harness::parsing::{parse, ChunkedInput};
use crate::harness::ASCII_COMPATIBLE_ENCODINGS;
use cool_thing::parser::TextType;
use cool_thing::token::{
    Attribute, AttributeValidationError, Token, TokenCaptureFlags, TokenFactory,
};
use cool_thing::transform_stream::Serialize;
use encoding_rs::{Encoding, EUC_JP, UTF_8};

fn serialize_attr(attr: Attribute<'_>, encoding: &'static Encoding) -> String {
    let mut bytes = Vec::new();

    attr.into_bytes(&mut |c| bytes.extend_from_slice(&c));

    encoding.decode(&bytes).0.into_owned()
}

test_fixture!("Attributes", {
    test("Empty name", {
        let factory = TokenFactory::new(UTF_8);

        [
            factory.new_attribute("", "").expect_err("Error expected"),
            factory
                .new_attribute("foo", "")
                .unwrap()
                .set_name("")
                .expect_err("Error expected"),
        ]
        .into_iter()
        .for_each(|err| {
            assert_eq!(
                err.downcast_ref::<AttributeValidationError>().cloned(),
                Some(AttributeValidationError::EmptyName)
            );
        });
    });

    test("Forbidden characters in name", {
        let factory = TokenFactory::new(UTF_8);

        [' ', '\n', '\r', '\t', '\x0C', '/', '>', '=']
            .into_iter()
            .for_each(|&ch| {
                let name = format!("foo{}bar", ch);

                [
                    factory
                        .new_attribute(&name, "")
                        .expect_err("Error expected"),
                    factory
                        .new_attribute("foo", "")
                        .unwrap()
                        .set_name(&name)
                        .expect_err("Error expected"),
                ]
                .into_iter()
                .for_each(|err| {
                    assert_eq!(
                        err.downcast_ref::<AttributeValidationError>().cloned(),
                        Some(AttributeValidationError::ForbiddenCharacterInName(ch))
                    );
                });
            });
    });

    test("Encoding unmappable characters in name", {
        let factory = TokenFactory::new(EUC_JP);

        [
            factory
                .new_attribute("foo\u{00F8}bar", "")
                .expect_err("Error expected"),
            factory
                .new_attribute("foo", "")
                .unwrap()
                .set_name("foo\u{00F8}bar")
                .expect_err("Error expected"),
        ]
        .into_iter()
        .for_each(|err| {
            assert_eq!(
                err.downcast_ref::<AttributeValidationError>().cloned(),
                Some(AttributeValidationError::UnencodableCharacterInName)
            );
        });
    });

    test("Construction and mutation", {
        ASCII_COMPATIBLE_ENCODINGS.iter().for_each(|enc| {
            let factory = TokenFactory::new(enc);

            let mut attr = factory
                .new_attribute("Foo", "Bar")
                .expect("Attribute should be constructed");

            assert_eq!(attr.name(), "foo", "Encoding: {}", enc.name());
            assert_eq!(attr.value(), "Bar", "Encoding: {}", enc.name());

            attr.set_name("TeSt").expect("Name setter shouldn't error");
            attr.set_value("NewValue42");

            assert_eq!(attr.name(), "test", "Encoding: {}", enc.name());
            assert_eq!(attr.value(), "NewValue42", "Encoding: {}", enc.name());
        });
    });

    test("Serialization from parts", {
        ASCII_COMPATIBLE_ENCODINGS.iter().for_each(|enc| {
            let attr = TokenFactory::new(enc)
                .new_attribute("FooBar", r#"'Test1' "Test2" 'Test3'"#)
                .expect("Attribute should be constructed");

            assert_eq!(
                serialize_attr(attr, enc),
                r#"FooBar="'Test1' &quot;Test2&quot; 'Test3'""#
            );
        });
    });

    test("Parsed attribute serialization", {
        ASCII_COMPATIBLE_ENCODINGS.iter().for_each(|enc| {
            let src = [
                "<a attr1>",
                r#"<a attr2='foo"bar'>"#,
                r#"<a attr3="foo'bar">"#,
                "<a attr4=baz>",
            ];

            let serialized: Vec<_> = src
                .into_iter()
                .map(|&input| {
                    let mut input: ChunkedInput = String::from(input).into();
                    let mut res = Vec::new();

                    input
                        .init(enc)
                        .expect("Input should be initialized successfully");

                    parse(
                        &input,
                        TokenCaptureFlags::START_TAGS,
                        TextType::Data,
                        None,
                        Box::new(|token| match token {
                            Token::StartTag(start_tag) => {
                                let attr = &start_tag.attributes()[0];

                                let test_cases = vec![
                                    attr.to_owned(),
                                    {
                                        let mut attr = attr.to_owned();

                                        attr.set_name("changed_name")
                                            .expect("Name setter shouldn't error");

                                        attr
                                    },
                                    {
                                        let mut attr = attr.to_owned();

                                        attr.set_value("changed_value");

                                        attr
                                    },
                                ];

                                test_cases
                                    .into_iter()
                                    .for_each(|attr| res.push(serialize_attr(attr, enc)));
                            }
                            _ => unreachable!("Start tag expected"),
                        }),
                    )
                    .expect("Input should be parsed successfully");

                    res
                })
                .collect();

            assert_eq!(
                serialized,
                [
                    ["attr1", r#"changed_name="""#, r#"attr1="changed_value""#],
                    [
                        r#"attr2='foo"bar'"#,
                        r#"changed_name="foo&quot;bar""#,
                        r#"attr2="changed_value""#
                    ],
                    [
                        r#"attr3="foo'bar""#,
                        r#"changed_name="foo'bar""#,
                        r#"attr3="changed_value""#
                    ],
                    [
                        "attr4=baz",
                        r#"changed_name="baz""#,
                        r#"attr4="changed_value""#
                    ]
                ]
            );
        });
    });
});
