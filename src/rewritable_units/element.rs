use super::{Attribute, AttributeNameError, ContentType, EndTag, Mutations, StartTag};
use crate::base::Bytes;
use crate::rewriter::EndTagHandler;
use encoding_rs::Encoding;
use std::any::Any;
use std::fmt::{self, Debug};

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum TagNameError {
    #[fail(display = "Tag name can't be empty.")]
    Empty,
    #[fail(display = "First character of the tag name should be an ASCII alphabetical character.")]
    InvalidFirstCharacter,
    #[fail(display = "{:?} character is forbidden in the tag name", _0)]
    ForbiddenCharacter(char),
    #[fail(display = "The tag name contains a character that can't \
                      be represented in the document's character encoding.")]
    UnencodableCharacter,
}

pub struct Element<'r, 't> {
    start_tag: &'r mut StartTag<'t>,
    end_tag_mutations: Option<Mutations>,
    modified_end_tag_name: Option<Bytes<'static>>,
    can_have_content: bool,
    should_remove_content: bool,
    encoding: &'static Encoding,
    user_data: Box<dyn Any>,
}

impl<'r, 't> Element<'r, 't> {
    pub(crate) fn new(start_tag: &'r mut StartTag<'t>, can_have_content: bool) -> Self {
        let encoding = start_tag.encoding();

        Element {
            start_tag,
            end_tag_mutations: None,
            modified_end_tag_name: None,
            can_have_content,
            should_remove_content: false,
            encoding,
            user_data: Box::new(()),
        }
    }

    fn tag_name_bytes_from_str(&self, name: &str) -> Result<Bytes<'static>, TagNameError> {
        match name.chars().nth(0) {
            Some(ch) if !ch.is_ascii_alphabetic() => Err(TagNameError::InvalidFirstCharacter),
            Some(_) => {
                if let Some(ch) = name.chars().find(|&ch| match ch {
                    ' ' | '\n' | '\r' | '\t' | '\x0C' | '/' | '>' => true,
                    _ => false,
                }) {
                    Err(TagNameError::ForbiddenCharacter(ch))
                } else {
                    // NOTE: if character can't be represented in the given
                    // encoding then encoding_rs replaces it with a numeric
                    // character reference. Character references are not
                    // supported in tag names, so we need to bail.
                    match Bytes::from_str_without_replacements(name, self.encoding) {
                        Ok(name) => Ok(name.into_owned()),
                        Err(_) => Err(TagNameError::UnencodableCharacter),
                    }
                }
            }
            None => Err(TagNameError::Empty),
        }
    }

    #[inline]
    fn remove_content(&mut self) {
        self.start_tag.mutations.content_after.clear();
        self.end_tag_mutations_mut().content_before.clear();
        self.should_remove_content = true;
    }

    #[inline]
    fn end_tag_mutations_mut(&mut self) -> &mut Mutations {
        let encoding = self.encoding;

        self.end_tag_mutations
            .get_or_insert_with(|| Mutations::new(encoding))
    }

    #[inline]
    pub fn tag_name(&self) -> String {
        self.start_tag.name()
    }

    #[inline]
    pub fn set_tag_name(&mut self, name: &str) -> Result<(), TagNameError> {
        let name = self.tag_name_bytes_from_str(name)?;

        if self.can_have_content {
            self.modified_end_tag_name = Some(name.clone());
        }

        self.start_tag.set_name(name);

        Ok(())
    }

    #[inline]
    pub fn namespace_uri(&self) -> &'static str {
        self.start_tag.namespace_uri()
    }

    #[inline]
    pub fn attributes(&self) -> &[Attribute<'t>] {
        self.start_tag.attributes()
    }

    #[inline]
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        let name = name.to_ascii_lowercase();

        self.attributes().iter().find_map(|attr| {
            if attr.name() == name {
                Some(attr.value())
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn has_attribute(&self, name: &str) -> bool {
        let name = name.to_ascii_lowercase();

        self.attributes().iter().any(|attr| attr.name() == name)
    }

    #[inline]
    pub fn set_attribute(&mut self, name: &str, value: &str) -> Result<(), AttributeNameError> {
        self.start_tag.set_attribute(name, value)
    }

    #[inline]
    pub fn remove_attribute(&mut self, name: &str) {
        self.start_tag.remove_attribute(name);
    }

    #[inline]
    pub fn before(&mut self, content: &str, content_type: ContentType) {
        self.start_tag.mutations.before(content, content_type);
    }

    #[inline]
    pub fn after(&mut self, content: &str, content_type: ContentType) {
        if self.can_have_content {
            self.end_tag_mutations_mut().after(content, content_type);
        } else {
            self.start_tag.mutations.after(content, content_type);
        }
    }

    #[inline]
    pub fn prepend(&mut self, content: &str, content_type: ContentType) {
        self.start_tag.mutations.after(content, content_type);
    }

    #[inline]
    pub fn append(&mut self, content: &str, content_type: ContentType) {
        if self.can_have_content {
            self.end_tag_mutations_mut().before(content, content_type);
        }
    }

    #[inline]
    pub fn set_inner_content(&mut self, content: &str, content_type: ContentType) {
        if self.can_have_content {
            self.remove_content();
            self.start_tag.mutations.after(content, content_type);
        }
    }

    #[inline]
    pub fn replace(&mut self, content: &str, content_type: ContentType) {
        self.start_tag.mutations.replace(content, content_type);

        if self.can_have_content {
            self.remove_content();
            self.end_tag_mutations_mut().remove();
        }
    }

    #[inline]
    pub fn remove(&mut self) {
        self.start_tag.mutations.remove();

        if self.can_have_content {
            self.remove_content();
            self.end_tag_mutations_mut().remove();
        }
    }

    #[inline]
    pub fn remove_and_keep_content(&mut self) {
        self.start_tag.mutations.remove();

        if self.can_have_content {
            self.end_tag_mutations_mut().remove();
        }
    }

    #[inline]
    pub fn removed(&self) -> bool {
        self.start_tag.mutations.removed()
    }

    #[inline]
    pub(crate) fn should_remove_content(&self) -> bool {
        self.should_remove_content
    }

    pub(crate) fn into_end_tag_handler(self) -> Option<EndTagHandler<'static>> {
        let end_tag_mutations = self.end_tag_mutations;
        let modified_end_tag_name = self.modified_end_tag_name;

        if end_tag_mutations.is_some() || modified_end_tag_name.is_some() {
            // NOTE: Rc<RefCell<FnOnce>> is not callable in Rust, because it will
            // require consumption of the inner value. To workaround it, we wrap
            // FnOnce into FnMut and use runtime check to ensure that it has been
            // called only once.
            let mut wrap = Some(move |end_tag: &mut EndTag| {
                if let Some(name) = modified_end_tag_name {
                    end_tag.set_name(name);
                }

                if let Some(mutations) = end_tag_mutations {
                    end_tag.mutations = mutations;
                }
            });

            Some(Box::new(move |end_tag: &mut EndTag| {
                (wrap.take().expect("FnOnce called more than once"))(end_tag);
                Ok(())
            }))
        } else {
            None
        }
    }
}

impl_user_data!(Element<'_, '_>);

impl Debug for Element<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Element")
            .field("tag_name", &self.tag_name())
            .field("attributes", &self.attributes())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::rewritable_units::test_utils::*;
    use crate::test_utils::ASCII_COMPATIBLE_ENCODINGS;
    use crate::*;
    use encoding_rs::{Encoding, EUC_JP, UTF_8};

    fn rewrite_element(
        html: &str,
        encoding: &'static Encoding,
        selector: &str,
        mut handler: impl FnMut(&mut Element),
    ) -> String {
        let mut handler_called = false;

        let output = rewrite_html(
            html,
            encoding,
            vec![
                (
                    &selector.parse().unwrap(),
                    ElementContentHandlers::default().element(|el| {
                        handler_called = true;
                        handler(el);
                        Ok(())
                    }),
                ),
                (
                    // NOTE: used to test inner content removal
                    &"inner-remove-me".parse().unwrap(),
                    ElementContentHandlers::default().element(|el| {
                        el.before("[before: should be removed]", ContentType::Text);
                        el.after("[after: should be removed]", ContentType::Text);
                        el.append("[append: should be removed]", ContentType::Text);
                        el.before("[before: should be removed]", ContentType::Text);
                        Ok(())
                    }),
                ),
            ],
            vec![],
        );

        assert!(handler_called);

        output
    }

    #[test]
    fn empty_tag_name() {
        rewrite_element("<div>", UTF_8, "div", |el| {
            let err = el.set_tag_name("").unwrap_err();

            assert_eq!(err, TagNameError::Empty);
        });
    }

    #[test]
    fn forbidden_characters_in_tag_name() {
        rewrite_element("<div>", UTF_8, "div", |el| {
            for &ch in &[' ', '\n', '\r', '\t', '\x0C', '/', '>'] {
                let err = el.set_tag_name(&format!("foo{}bar", ch)).unwrap_err();

                assert_eq!(err, TagNameError::ForbiddenCharacter(ch));
            }
        });
    }

    #[test]
    fn encoding_unmappable_chars_in_tag_name() {
        rewrite_element("<div>", EUC_JP, "div", |el| {
            let err = el.set_tag_name("foo\u{00F8}bar").unwrap_err();

            assert_eq!(err, TagNameError::UnencodableCharacter);
        });
    }

    #[test]
    fn invalid_first_char_of_tag_name() {
        rewrite_element("<div>", UTF_8, "div", |el| {
            let err = el.set_tag_name("1foo").unwrap_err();

            assert_eq!(err, TagNameError::InvalidFirstCharacter);
        });
    }

    #[test]
    fn namespace_uri() {
        rewrite_element("<script></script>", UTF_8, "script", |el| {
            assert_eq!(el.namespace_uri(), "http://www.w3.org/1999/xhtml");
        });

        rewrite_element("<svg><script></script></svg>", UTF_8, "script", |el| {
            assert_eq!(el.namespace_uri(), "http://www.w3.org/2000/svg");
        });

        rewrite_element(
            "<svg><foreignObject><script></script></foreignObject></svg>",
            UTF_8,
            "script",
            |el| {
                assert_eq!(el.namespace_uri(), "http://www.w3.org/1999/xhtml");
            },
        );

        rewrite_element("<math><script></script></math>", UTF_8, "script", |el| {
            assert_eq!(el.namespace_uri(), "http://www.w3.org/1998/Math/MathML");
        });
    }

    #[test]
    fn empty_attr_name() {
        rewrite_element("<div>", UTF_8, "div", |el| {
            let err = el.set_attribute("", "").unwrap_err();

            assert_eq!(err, AttributeNameError::Empty);
        });
    }

    #[test]
    fn forbidden_characters_in_attr_name() {
        rewrite_element("<div>", UTF_8, "div", |el| {
            for &ch in &[' ', '\n', '\r', '\t', '\x0C', '/', '>', '='] {
                let err = el.set_attribute(&format!("foo{}bar", ch), "").unwrap_err();

                assert_eq!(err, AttributeNameError::ForbiddenCharacter(ch));
            }
        });
    }

    #[test]
    fn encoding_unmappable_character_in_attr_name() {
        rewrite_element("<div>", EUC_JP, "div", |el| {
            let err = el.set_attribute("foo\u{00F8}bar", "").unwrap_err();

            assert_eq!(err, AttributeNameError::UnencodableCharacter);
        });
    }

    #[test]
    fn tag_name_getter_and_setter() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<Foo><div><span></span></div></foo>", enc, "foo", |el| {
                assert_eq!(el.tag_name(), "foo", "Encoding: {}", enc.name());

                el.set_tag_name("BaZ").unwrap();

                assert_eq!(el.tag_name(), "baz", "Encoding: {}", enc.name());
            });

            assert_eq!(output, "<BaZ><div><span></span></div></BaZ>");
        }
    }

    #[test]
    fn attribute_list() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            rewrite_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, "foo", |el| {
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
    }

    #[test]
    fn get_attrs() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            rewrite_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, "foo", |el| {
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
    }

    #[test]
    fn has_attr() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            rewrite_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, "foo", |el| {
                assert!(el.has_attribute("FOo1"), "Encoding: {}", enc.name());
                assert!(el.has_attribute("foo1"), "Encoding: {}", enc.name());
                assert!(el.has_attribute("FOO2"), "Encoding: {}", enc.name());
                assert!(!el.has_attribute("foo3"), "Encoding: {}", enc.name());
            });
        }
    }

    #[test]
    fn set_attr() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            rewrite_element("<div>", enc, "div", |el| {
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
    }

    #[test]
    fn remove_attr() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            rewrite_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, "foo", |el| {
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
    }

    #[test]
    fn insert_content_before() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Hi</span></div>", enc, "span", |el| {
                el.before("<img>", ContentType::Html);
                el.before("<img>", ContentType::Text);
            });

            assert_eq!(output, "<div><img>&lt;img&gt;<span>Hi</span></div>");
        }
    }

    #[test]
    fn prepend_content() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Hi</span></div>", enc, "span", |el| {
                el.prepend("<img>", ContentType::Html);
                el.prepend("<img>", ContentType::Text);
            });

            assert_eq!(output, "<div><span>&lt;img&gt;<img>Hi</span></div>");
        }
    }

    #[test]
    fn append_content() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Hi</span></div>", enc, "span", |el| {
                el.append("<img>", ContentType::Html);
                el.append("<img>", ContentType::Text);
            });

            assert_eq!(output, "<div><span>Hi<img>&lt;img&gt;</span></div>");
        }
    }

    #[test]
    fn insert_content_after() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Hi</span></div>", enc, "span", |el| {
                el.after("<img>", ContentType::Html);
                el.after("<img>", ContentType::Text);
            });

            assert_eq!(output, "<div><span>Hi</span>&lt;img&gt;<img></div>");
        }
    }

    #[test]
    fn set_content_after() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element(
                "<div><span>Hi<inner-remove-me>Remove</inner-remove-me></span></div>",
                enc,
                "span",
                |el| {
                    el.prepend("<prepended>", ContentType::Html);
                    el.append("<appended>", ContentType::Html);
                    el.set_inner_content("<img>", ContentType::Html);
                    el.set_inner_content("<img>", ContentType::Text);
                },
            );

            assert_eq!(output, "<div><span>&lt;img&gt;</span></div>");

            let output = rewrite_element(
                "<div><span>Hi<inner-remove-me>Remove</inner-remove-me></span></div>",
                enc,
                "span",
                |el| {
                    el.prepend("<prepended>", ContentType::Html);
                    el.append("<appended>", ContentType::Html);
                    el.set_inner_content("<img>", ContentType::Text);
                    el.set_inner_content("<img>", ContentType::Html);
                },
            );

            assert_eq!(output, "<div><span><img></span></div>");
        }
    }

    #[test]
    fn replace() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element(
                "<div><span>Hi<inner-remove-me>Remove</inner-remove-me></span></div>",
                enc,
                "span",
                |el| {
                    el.prepend("<prepended>", ContentType::Html);
                    el.append("<appended>", ContentType::Html);
                    el.replace("<img>", ContentType::Html);
                    el.replace("<img>", ContentType::Text);

                    assert!(el.removed());
                },
            );

            assert_eq!(output, "<div>&lt;img&gt;</div>");

            let output = rewrite_element(
                "<div><span>Hi<inner-remove-me>Remove</inner-remove-me></span></div>",
                enc,
                "span",
                |el| {
                    el.prepend("<prepended>", ContentType::Html);
                    el.append("<appended>", ContentType::Html);
                    el.replace("<img>", ContentType::Text);
                    el.replace("<img>", ContentType::Html);

                    assert!(el.removed());
                },
            );

            assert_eq!(output, "<div><img></div>");
        }
    }

    #[test]
    fn remove() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element(
                "<div><span>Hi<inner-remove-me>Remove</inner-remove-me></span></div>",
                enc,
                "span",
                |el| {
                    el.prepend("<prepended>", ContentType::Html);
                    el.append("<appended>", ContentType::Html);
                    el.remove();

                    assert!(el.removed());
                },
            );

            assert_eq!(output, "<div></div>");
        }
    }

    #[test]
    fn remove_with_unfinished_end_tag() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Heello</span  ", enc, "span", |el| {
                el.remove();

                assert!(el.removed());
            });

            assert_eq!(output, "<div>");
        }
    }

    #[test]
    fn remove_and_keep_content() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Hi</span></div>", enc, "span", |el| {
                el.prepend("<prepended>", ContentType::Html);
                el.append("<appended>", ContentType::Html);
                el.remove_and_keep_content();

                assert!(el.removed());
            });

            assert_eq!(output, "<div><prepended>Hi<appended></div>");
        }
    }

    #[test]
    fn multiple_consequent_removes() {
        let output = rewrite_html(
            "<div><span>42</span></div><h1>Hello</h1><h2>Hello2</h2>",
            UTF_8,
            vec![
                (
                    &"div".parse().unwrap(),
                    ElementContentHandlers::default().element(|el| {
                        el.replace("hey & ya", ContentType::Html);
                        Ok(())
                    }),
                ),
                (
                    &"h1".parse().unwrap(),
                    ElementContentHandlers::default().element(|el| {
                        el.remove();
                        Ok(())
                    }),
                ),
                (
                    &"h2".parse().unwrap(),
                    ElementContentHandlers::default().element(|el| {
                        el.remove_and_keep_content();
                        Ok(())
                    }),
                ),
            ],
            vec![],
        );

        assert_eq!(output, "hey & yaHello2");
    }

    #[test]
    fn void_element() {
        let output = rewrite_element("<img><span>Hi</span></img>", UTF_8, "img", |el| {
            el.after("<!--after-->", ContentType::Html);
            el.set_tag_name("img-foo").unwrap();
        });

        assert_eq!(output, "<img-foo><!--after--><span>Hi</span></img>");
    }

    #[test]
    fn self_closing_element() {
        let output = rewrite_element("<svg><foo/>Hi</foo></svg>", UTF_8, "foo", |el| {
            el.after("<!--after-->", ContentType::Html);
            el.set_tag_name("bar").unwrap();
        });

        assert_eq!(output, "<svg><bar/><!--after-->Hi</foo></svg>");
    }

    #[test]
    fn user_data() {
        rewrite_element("<div><span>Hi</span></div>", UTF_8, "span", |el| {
            el.set_user_data(42usize);

            assert_eq!(*el.user_data().downcast_ref::<usize>().unwrap(), 42usize);

            *el.user_data_mut().downcast_mut::<usize>().unwrap() = 1337usize;

            assert_eq!(*el.user_data().downcast_ref::<usize>().unwrap(), 1337usize);
        });
    }

    mod serialization {
        use super::*;

        const HTML: &str = r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4></a>"#;
        const SELECTOR: &str = "a";

        macro_rules! test {
            ($handler:expr, $expected:expr) => {
                for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
                    assert_eq!(rewrite_element(HTML, enc, SELECTOR, $handler), $expected);
                }
            };
        }

        #[test]
        fn parsed() {
            test!(
                |_| {},
                r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4></a>"#
            );
        }

        #[test]
        fn modified_name() {
            test!(
                |el| {
                    el.set_tag_name("div").unwrap();
                },
                r#"<div a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4></div>"#
            );
        }

        #[test]
        fn modified_single_quoted_attr() {
            test!(
                |el| {
                    el.set_attribute("a2", "foo ' bar ' baz42").unwrap();
                },
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz42" a3=foo/bar a4></a>"#
            );
        }

        #[test]
        fn modified_double_quoted_attr() {
            test!(
                |el| {
                    el.set_attribute("a2", "foo ' bar ' baz42").unwrap();
                },
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz42" a3=foo/bar a4></a>"#
            );
        }

        #[test]
        fn modified_unquoted_attr() {
            test!(
                |el| {
                    el.set_attribute("a3", "foo/bar42").unwrap();
                },
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3="foo/bar42" a4></a>"#
            );
        }

        #[test]
        fn set_value_for_attr_without_value() {
            test!(
                |el| {
                    el.set_attribute("a4", "42").unwrap();
                },
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4="42"></a>"#
            );
        }

        #[test]
        fn add_attr() {
            test!(
            |el| {
                el.set_attribute("a5", r#"42'"42"#).unwrap();
            },
            r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4 a5="42'&quot;42"></a>"#
        );
        }

        #[test]
        fn self_closing_flag() {
            // NOTE: we should add space between valueless attr and self-closing slash
            // during serialization. Otherwise, it will be interpreted as a part of the
            // attribute name.
            let mut output = rewrite_element("<img a1=42 a2 />", UTF_8, "img", |el| {
                el.set_attribute("a1", "foo").unwrap();
            });

            assert_eq!(output, r#"<img a1="foo" a2 />"#);

            // NOTE: but we shouldn't add space if there are no attributes.
            output = rewrite_element("<img a1 />", UTF_8, "img", |el| {
                el.remove_attribute("a1");
            });

            assert_eq!(output, r#"<img/>"#);
        }

        #[test]
        fn remove_non_existent_attr() {
            test!(
                |el| {
                    el.remove_attribute("a5");
                },
                r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4></a>"#
            );
        }

        #[test]
        fn without_attrs() {
            test!(
                |el| {
                    for name in &["a1", "a2", "a3", "a4"] {
                        el.remove_attribute(name);
                    }
                },
                "<a></a>"
            );
        }

        #[test]
        fn with_before_and_prepend() {
            test!(
                |el| {
                    el.before("<span>", ContentType::Text);
                    el.before("<div>Hey</div>", ContentType::Html);
                    el.before("<foo>", ContentType::Html);
                    el.prepend("</foo>", ContentType::Html);
                    el.prepend("<!-- 42 -->", ContentType::Html);
                    el.prepend("<foo & bar>", ContentType::Text);
                },
                concat!(
                    "&lt;span&gt;<div>Hey</div><foo>",
                    r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
                    "&lt;foo &amp; bar&gt;<!-- 42 --></foo>",
                    "</a>"
                )
            );
        }

        #[test]
        fn with_after_and_append() {
            test!(
                |el| {
                    el.append("<span>", ContentType::Text);
                    el.append("<div>Hey</div>", ContentType::Html);
                    el.append("<foo>", ContentType::Html);
                    el.after("</foo>", ContentType::Html);
                    el.after("<!-- 42 -->", ContentType::Html);
                    el.after("<foo & bar>", ContentType::Text);
                },
                concat!(
                    r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
                    "&lt;span&gt;<div>Hey</div><foo>",
                    "</a>",
                    "&lt;foo &amp; bar&gt;<!-- 42 --></foo>",
                )
            );
        }

        #[test]
        fn removed() {
            test!(
                |el| {
                    assert!(!el.removed());

                    el.remove();

                    assert!(el.removed());

                    el.before("<before>", ContentType::Html);
                    el.after("<after>", ContentType::Html);
                },
                "<before><after>"
            );
        }

        #[test]
        fn replaced_with_text() {
            test!(
                |el| {
                    el.before("<before>", ContentType::Html);
                    el.after("<after>", ContentType::Html);

                    assert!(!el.removed());

                    el.replace("<div></div>", ContentType::Html);
                    el.replace("<!--42-->", ContentType::Html);
                    el.replace("<foo & bar>", ContentType::Text);

                    assert!(el.removed());
                },
                "<before>&lt;foo &amp; bar&gt;<after>"
            );
        }

        #[test]
        fn replaced_with_html() {
            test!(
                |el| {
                    el.before("<before>", ContentType::Html);
                    el.after("<after>", ContentType::Html);

                    assert!(!el.removed());

                    el.replace("<div></div>", ContentType::Html);
                    el.replace("<!--42-->", ContentType::Html);
                    el.replace("<foo & bar>", ContentType::Html);

                    assert!(el.removed());
                },
                "<before><foo & bar><after>"
            );
        }
    }

}
