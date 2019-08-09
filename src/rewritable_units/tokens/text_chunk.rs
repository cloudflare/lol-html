use super::{Mutations, Token};
use crate::base::Bytes;
use crate::html::TextType;
use encoding_rs::Encoding;
use std::any::Any;
use std::borrow::Cow;
use std::fmt::{self, Debug};

pub struct TextChunk<'i> {
    text: Cow<'i, str>,
    text_type: TextType,
    last_in_text_node: bool,
    encoding: &'static Encoding,
    mutations: Mutations,
    user_data: Box<dyn Any>,
}

impl<'i> TextChunk<'i> {
    pub(super) fn new_token(
        text: &'i str,
        text_type: TextType,
        last_in_text_node: bool,
        encoding: &'static Encoding,
    ) -> Token<'i> {
        Token::TextChunk(TextChunk {
            text: text.into(),
            text_type,
            last_in_text_node,
            encoding,
            mutations: Mutations::new(encoding),
            user_data: Box::new(()),
        })
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &*self.text
    }

    #[inline]
    pub fn text_type(&self) -> TextType {
        self.text_type
    }

    #[inline]
    pub fn last_in_text_node(&self) -> bool {
        self.last_in_text_node
    }

    #[inline]
    fn raw(&self) -> Option<&Bytes> {
        None
    }

    #[inline]
    fn serialize_from_parts(&self, output_handler: &mut dyn FnMut(&[u8])) {
        if !self.text.is_empty() {
            output_handler(&Bytes::from_str(&self.text, self.encoding));
        }
    }
}

inject_mutation_api!(TextChunk);
impl_serialize!(TextChunk);
impl_user_data!(TextChunk<'_>);

impl Debug for TextChunk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TextChunk")
            .field("text", &self.as_str())
            .field("last_in_text_node", &self.last_in_text_node())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::rewritable_units::test_utils::*;
    use crate::test_utils::ASCII_COMPATIBLE_ENCODINGS;
    use crate::*;
    use encoding_rs::{Encoding, UTF_8};

    fn rewrite_text_chunk(
        html: &str,
        encoding: &'static Encoding,
        mut handler: impl FnMut(&mut TextChunk),
    ) -> String {
        let mut handler_called = false;

        let output = rewrite_html(
            html,
            encoding,
            vec![],
            vec![DocumentContentHandlers::default().text(|c| {
                handler_called = true;
                handler(c);
                Ok(())
            })],
        );

        assert!(handler_called);

        output
    }

    #[test]
    fn user_data() {
        rewrite_text_chunk("foo", UTF_8, |c| {
            c.set_user_data(42usize);

            assert_eq!(*c.user_data().downcast_ref::<usize>().unwrap(), 42usize);

            *c.user_data_mut().downcast_mut::<usize>().unwrap() = 1337usize;

            assert_eq!(*c.user_data().downcast_ref::<usize>().unwrap(), 1337usize);
        });
    }

    mod serialization {
        use super::*;

        const HTML: &str =
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor \
             incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud \
             exercitation & ullamco laboris nisi ut aliquip ex ea commodo > consequat.";

        macro_rules! test {
            ($handler:expr, $expected:expr) => {
                for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
                    assert_eq!(rewrite_text_chunk(HTML, enc, $handler), $expected);
                }
            };
        }

        macro_rules! skip_eof_chunk {
            ($c:ident) => {
                if $c.last_in_text_node() {
                    assert!($c.as_str().is_empty());
                    return;
                }
            };
        }

        #[test]
        fn parsed() {
            test!(|_| {}, HTML);
        }

        #[test]
        fn with_prepends_and_appends() {
            test!(
                |c| {
                    skip_eof_chunk!(c);
                    c.before("<span>", ContentType::Text);
                    c.before("<div>Hey</div>", ContentType::Html);
                    c.before("<foo>", ContentType::Html);
                    c.after("</foo>", ContentType::Html);
                    c.after("<!-- 42 -->", ContentType::Html);
                    c.after("<foo & bar>", ContentType::Text);
                },
                concat!(
                    "&lt;span&gt;<div>Hey</div><foo>",
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod \
                     tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim \
                     veniam, quis nostrud exercitation & ullamco laboris nisi ut aliquip \
                     ex ea commodo > consequat.",
                    "&lt;foo &amp; bar&gt;<!-- 42 --></foo>"
                )
            );
        }

        #[test]
        fn removed() {
            test!(
                |c| {
                    skip_eof_chunk!(c);
                    assert!(!c.removed());

                    c.remove();

                    assert!(c.removed());

                    c.before("<before>", ContentType::Html);
                    c.after("<after>", ContentType::Html);
                },
                "<before><after>"
            );
        }

        #[test]
        fn replaced_with_text() {
            test!(
                |c| {
                    skip_eof_chunk!(c);
                    c.before("<before>", ContentType::Html);
                    c.after("<after>", ContentType::Html);

                    assert!(!c.removed());

                    c.replace("<div></div>", ContentType::Html);
                    c.replace("<!--42-->", ContentType::Html);
                    c.replace("<foo & bar>", ContentType::Text);

                    assert!(c.removed());
                },
                "<before>&lt;foo &amp; bar&gt;<after>"
            );
        }

        #[test]
        fn replaced_with_html() {
            test!(
                |c| {
                    skip_eof_chunk!(c);
                    c.before("<before>", ContentType::Html);
                    c.after("<after>", ContentType::Html);

                    assert!(!c.removed());

                    c.replace("<div></div>", ContentType::Html);
                    c.replace("<!--42-->", ContentType::Html);
                    c.replace("<foo & bar>", ContentType::Html);

                    assert!(c.removed());
                },
                "<before><foo & bar><after>"
            );
        }
    }
}
