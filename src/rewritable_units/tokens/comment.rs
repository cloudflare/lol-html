use super::{Mutations, Token};
use crate::base::Bytes;
use encoding_rs::Encoding;
use std::any::Any;
use std::fmt::{self, Debug};

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum CommentTextError {
    #[fail(display = "Comment text shouldn't contain comment closing sequence (`-->`).")]
    CommentClosingSequence,
    #[fail(display = "Comment text contains a character that can't \
                      be represented in the document's character encoding.")]
    UnencodableCharacter,
}

pub struct Comment<'i> {
    text: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
    mutations: Mutations,
    user_data: Box<dyn Any>,
}

impl<'i> Comment<'i> {
    pub(super) fn new_token(
        text: Bytes<'i>,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Token<'i> {
        Token::Comment(Comment {
            text,
            raw: Some(raw),
            encoding,
            mutations: Mutations::new(encoding),
            user_data: Box::new(()),
        })
    }

    #[inline]
    pub fn text(&self) -> String {
        self.text.as_string(self.encoding)
    }

    #[inline]
    pub fn set_text(&mut self, text: &str) -> Result<(), CommentTextError> {
        if text.find("-->").is_some() {
            Err(CommentTextError::CommentClosingSequence)
        } else {
            // NOTE: if character can't be represented in the given
            // encoding then encoding_rs replaces it with a numeric
            // character reference. Character references are not
            // supported in comments, so we need to bail.
            match Bytes::from_str_without_replacements(text, self.encoding) {
                Ok(text) => {
                    self.text = text.into_owned();
                    self.raw = None;

                    Ok(())
                }
                Err(_) => Err(CommentTextError::UnencodableCharacter),
            }
        }
    }

    #[inline]
    fn raw(&self) -> Option<&Bytes> {
        self.raw.as_ref()
    }

    #[inline]
    fn serialize_from_parts(&self, output_handler: &mut dyn FnMut(&[u8])) {
        output_handler(b"<!--");
        output_handler(&self.text);
        output_handler(b"-->");
    }
}

inject_mutation_api!(Comment);
impl_serialize!(Comment);
impl_user_data!(Comment<'_>);

impl Debug for Comment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Comment")
            .field("text", &self.text())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::rewritable_units::test_utils::*;
    use crate::test_utils::ASCII_COMPATIBLE_ENCODINGS;
    use crate::*;
    use encoding_rs::{Encoding, EUC_JP, UTF_8};

    fn rewrite_comment(
        html: &str,
        encoding: &'static Encoding,
        mut handler: impl FnMut(&mut Comment),
    ) -> String {
        let mut handler_called = false;

        let output = rewrite_html(
            html,
            encoding,
            vec![],
            vec![DocumentContentHandlers::default().comments(|c| {
                handler_called = true;
                handler(c);
                Ok(())
            })],
        );

        assert!(handler_called);

        output
    }

    #[test]
    fn comment_closing_sequence_in_text() {
        rewrite_comment("<!-- foo -->", UTF_8, |c| {
            let err = c.set_text("foo -- bar --> baz").unwrap_err();

            assert_eq!(err, CommentTextError::CommentClosingSequence);
        });
    }

    #[test]
    fn encoding_unmappable_chars_in_text() {
        rewrite_comment("<!-- foo -->", EUC_JP, |c| {
            let err = c.set_text("foo\u{00F8}bar").unwrap_err();

            assert_eq!(err, CommentTextError::UnencodableCharacter);
        });
    }

    #[test]
    fn user_data() {
        rewrite_comment("<!-- foo -->", UTF_8, |c| {
            c.set_user_data(42usize);

            assert_eq!(*c.user_data().downcast_ref::<usize>().unwrap(), 42usize);

            *c.user_data_mut().downcast_mut::<usize>().unwrap() = 1337usize;

            assert_eq!(*c.user_data().downcast_ref::<usize>().unwrap(), 1337usize);
        });
    }

    mod serialization {
        use super::*;

        const HTML: &str = "<!-- foo -- bar -->";

        macro_rules! test {
            ($handler:expr, $expected:expr) => {
                for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
                    assert_eq!(rewrite_comment(HTML, enc, $handler), $expected);
                }
            };
        }

        #[test]
        fn parsed() {
            test!(|_| {}, "<!-- foo -- bar -->");
        }

        #[test]
        fn modified_text() {
            test!(
                |c| {
                    c.set_text("42 <!-").unwrap();
                },
                "<!--42 <!--->"
            );
        }

        #[test]
        fn with_prepends_and_appends() {
            test!(
                |c| {
                    c.before("<span>", ContentType::Text);
                    c.before("<div>Hey</div>", ContentType::Html);
                    c.before("<foo>", ContentType::Html);
                    c.after("</foo>", ContentType::Html);
                    c.after("<!-- 42 -->", ContentType::Html);
                    c.after("<foo & bar>", ContentType::Text);
                },
                concat!(
                    "&lt;span&gt;<div>Hey</div><foo><!-- foo -- bar -->",
                    "&lt;foo &amp; bar&gt;<!-- 42 --></foo>",
                )
            );
        }

        #[test]
        fn removed() {
            test!(
                |c| {
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
