mod handlers_dispatcher;
mod rewrite_controller;

#[macro_use]
pub(crate) mod settings;

use self::rewrite_controller::{ElementDescriptor, HtmlRewriteController};
pub use self::settings::*;
use crate::base::SharedEncoding;
use crate::memory::{MemoryLimitExceededError, SharedMemoryLimiter};
use crate::parser::ParsingAmbiguityError;
use crate::rewritable_units::{Element, IncompleteUtf8Resync};
use crate::transform_stream::*;
use encoding_rs::Encoding;
use mime::Mime;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::{self, Debug};
use thiserror::Error;

/// This is an encoding known to be ASCII-compatible.
///
/// Non-ASCII-compatible encodings (`UTF-16LE`, `UTF-16BE`, `ISO-2022-JP` and
/// `replacement`) are not supported by `lol_html`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AsciiCompatibleEncoding(&'static Encoding);

impl AsciiCompatibleEncoding {
    /// Returns `Some` if `Encoding` is ascii-compatible, or `None` otherwise.
    #[must_use]
    pub fn new(encoding: &'static Encoding) -> Option<Self> {
        encoding.is_ascii_compatible().then_some(Self(encoding))
    }

    fn from_mimetype(mime: &Mime) -> Option<Self> {
        let cs = mime.get_param("charset")?;
        Self::new(Encoding::for_label_no_replacement(cs.as_str().as_bytes())?)
    }

    /// Returns the most commonly used UTF-8 encoding.
    #[must_use]
    pub fn utf_8() -> Self {
        Self(encoding_rs::UTF_8)
    }

    #[must_use]
    pub(crate) fn get(self) -> &'static Encoding {
        self.0
    }
}

impl From<AsciiCompatibleEncoding> for &'static Encoding {
    fn from(ascii_enc: AsciiCompatibleEncoding) -> &'static Encoding {
        ascii_enc.0
    }
}

impl TryFrom<&'static Encoding> for AsciiCompatibleEncoding {
    type Error = ();

    fn try_from(enc: &'static Encoding) -> Result<Self, ()> {
        Self::new(enc).ok_or(())
    }
}

/// A compound error type that can be returned by [`write`] and [`end`] methods of the rewriter.
///
/// # Note
/// This error is unrecoverable. The rewriter instance will panic on attempt to use it after such an
/// error.
///
/// [`write`]: ../struct.HtmlRewriter.html#method.write
/// [`end`]: ../struct.HtmlRewriter.html#method.end
#[derive(Error, Debug)]
pub enum RewritingError {
    /// See [`MemoryLimitExceededError`].
    ///
    /// [`MemoryLimitExceededError`]: struct.MemoryLimitExceededError.html
    #[error("{0}")]
    MemoryLimitExceeded(MemoryLimitExceededError),

    /// See [`ParsingAmbiguityError`].
    ///
    /// [`ParsingAmbiguityError`]: struct.ParsingAmbiguityError.html
    #[error("{0}")]
    ParsingAmbiguity(ParsingAmbiguityError),

    /// An error that was propagated from one of the content handlers.
    #[error("{0}")]
    ContentHandlerError(Box<dyn StdError + Send + Sync + 'static>),
}

/// A streaming HTML rewriter.
///
/// # Example
/// ```
/// use lol_html::{element, HtmlRewriter, Settings};
///
/// let mut output = vec![];
///
/// {
///     let mut rewriter = HtmlRewriter::new(
///         Settings {
///             element_content_handlers: vec![
///                 // Rewrite insecure hyperlinks
///                 element!("a[href]", |el| {
///                     let href = el
///                         .get_attribute("href")
///                         .unwrap()
///                         .replace("http:", "https:");
///
///                     el.set_attribute("href", &href).unwrap();
///
///                     Ok(())
///                 })
///             ],
///             ..Settings::new()
///         },
///         |c: &[u8]| output.extend_from_slice(c)
///     );
///
///     rewriter.write(b"<div><a href=").unwrap();
///     rewriter.write(b"http://example.com>").unwrap();
///     rewriter.write(b"</a></div>").unwrap();
///     rewriter.end().unwrap();
/// }
///
/// assert_eq!(
///     String::from_utf8(output).unwrap(),
///     r#"<div><a href="https://example.com"></a></div>"#
/// );
/// ```
pub struct HtmlRewriter<'h, O: OutputSink, H: HandlerTypes = LocalHandlerTypes> {
    stream: TransformStream<HtmlRewriteController<'h, H>, O>,
    poisoned: bool,
}

macro_rules! guarded {
    ($self:ident, $expr:expr) => {{
        assert!(
            !$self.poisoned,
            "Attempt to use the HtmlRewriter after a fatal error."
        );

        let res = $expr;

        if res.is_err() {
            $self.poisoned = true;
        }

        res
    }};
}

impl<'h, O: OutputSink, H: HandlerTypes> HtmlRewriter<'h, O, H> {
    /// Constructs a new rewriter with the provided `settings` that writes
    /// the output to the `output_sink`.
    ///
    /// # Note
    ///
    /// For the convenience the [`OutputSink`] trait is implemented for closures.
    ///
    /// [`OutputSink`]: trait.OutputSink.html
    pub fn new<'s>(settings: Settings<'h, 's, H>, output_sink: O) -> Self {
        let preallocated_parsing_buffer_size =
            settings.memory_settings.preallocated_parsing_buffer_size;
        let graceful_bail_out_on_memory_limit_exceeded = settings
            .memory_settings
            .graceful_bail_out_on_memory_limit_exceeded;
        let graceful_bail_out_on_content_handler_error =
            settings.graceful_bail_out_on_content_handler_error;
        let strict = settings.strict;

        let encoding = settings.encoding;
        let next_encoding = SharedEncoding::default();

        let memory_limiter =
            SharedMemoryLimiter::new(settings.memory_settings.max_allowed_memory_usage);

        let stream = TransformStream::new(TransformStreamSettings {
            transform_controller: HtmlRewriteController::from_settings(
                settings,
                &memory_limiter,
                &next_encoding,
            ),
            output_sink,
            preallocated_parsing_buffer_size,
            memory_limiter,
            encoding,
            next_encoding,
            strict,
            graceful_bail_out_on_memory_limit_exceeded,
            graceful_bail_out_on_content_handler_error,
        });

        HtmlRewriter {
            stream,
            poisoned: false,
        }
    }

    /// Writes a chunk of input data to the rewriter.
    ///
    /// # Panics
    ///  * If previous invocation of the method returned a [`RewritingError`]
    ///    (these errors are unrecoverable).
    ///
    /// [`RewritingError`]: errors/enum.RewritingError.html
    /// [`end`]: struct.HtmlRewriter.html#method.end
    #[inline]
    pub fn write(&mut self, data: &[u8]) -> Result<(), RewritingError> {
        guarded!(self, self.stream.write(data))
    }

    /// Finalizes the rewriting process.
    ///
    /// Should be called once the last chunk of the input is written.
    ///
    /// # Panics
    ///  * If previous invocation of [`write`] returned a [`RewritingError`] (these errors
    ///    are unrecoverable).
    ///
    /// [`RewritingError`]: errors/enum.RewritingError.html
    /// [`write`]: struct.HtmlRewriter.html#method.write
    #[inline]
    pub fn end(mut self) -> Result<(), RewritingError> {
        guarded!(self, self.stream.end())
    }
}

// NOTE: this opaque Debug implementation is required to make
// `.unwrap()` and `.expect()` methods available on Result
// returned by the `HtmlRewriterBuilder.build()` method.
impl<O: OutputSink, H: HandlerTypes> Debug for HtmlRewriter<'_, O, H> {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HtmlRewriter")
    }
}

fn handler_adjust_charset_on_meta_tag<'h, H: HandlerTypes>(
    encoding: SharedEncoding,
) -> (Cow<'h, crate::Selector>, ElementContentHandlers<'h, H>) {
    // HTML5 allows encoding to be set only once
    let mut found = false;

    let handler = move |el: &mut Element<'_, '_, H>| {
        if found {
            return Ok(());
        }

        let charset = el.get_attribute("charset").and_then(|cs| {
            AsciiCompatibleEncoding::new(Encoding::for_label_no_replacement(cs.as_bytes())?)
        });

        let charset = charset.or_else(|| {
            el.get_attribute("http-equiv")
                .filter(|http_equiv| http_equiv.eq_ignore_ascii_case("Content-Type"))
                .and_then(|_| {
                    AsciiCompatibleEncoding::from_mimetype(
                        &el.get_attribute("content")?.parse::<Mime>().ok()?,
                    )
                })
        });

        if let Some(charset) = charset {
            found = true;
            let _ = encoding.set(charset);
        }

        Ok(())
    };

    let content_handlers = ElementContentHandlers {
        element: Some(H::new_element_handler(handler)),
        comments: None,
        text: None,
    };

    (Cow::Owned("meta".parse().unwrap()), content_handlers)
}

/// Rewrites given `html` string with the provided `settings`.
///
/// # Example
///
/// ```
/// use lol_html::{rewrite_str, element, RewriteStrSettings};
///
/// let element_content_handlers = vec![
///     // Rewrite insecure hyperlinks
///     element!("a[href]", |el| {
///         let href = el
///             .get_attribute("href")
///             .unwrap()
///             .replace("http:", "https:");
///
///          el.set_attribute("href", &href).unwrap();
///
///          Ok(())
///     })
/// ];
/// let output = rewrite_str(
///     r#"<div><a href="http://example.com"></a></div>"#,
///     RewriteStrSettings {
///         element_content_handlers,
///         ..RewriteStrSettings::new()
///     }
/// ).unwrap();
///
/// assert_eq!(output, r#"<div><a href="https://example.com"></a></div>"#);
/// ```
pub fn rewrite_str<'h, 's, H: HandlerTypes>(
    html: &str,
    settings: impl Into<Settings<'h, 's, H>>,
) -> Result<String, RewritingError> {
    let mut settings = settings.into();
    settings.adjust_charset_on_meta_tag = false;
    settings.encoding = AsciiCompatibleEncoding::utf_8();

    rewrite_str_utf8(html, settings)
}

#[inline(never)]
fn rewrite_str_utf8<H: HandlerTypes>(
    html: &str,
    settings: Settings<'_, '_, H>,
) -> Result<String, RewritingError> {
    let mut out = String::new();
    out.try_reserve(html.len())
        .map_err(|_| RewritingError::MemoryLimitExceeded(MemoryLimitExceededError))?;

    let mut resync = IncompleteUtf8Resync::new();
    let mut rewriter = HtmlRewriter::new(settings, |chunk: &[u8]| {
        if resync.write_utf8_chunk(chunk, |s| out.push_str(s)).is_err() {
            // this shouldn't fail, because we've got UTF-8 input and blocked encoding changes
            out.push('\u{FFFD}');
        }
    });

    rewriter.write(html.as_bytes())?;
    rewriter.end()?;

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::TextType;
    use crate::html_content::ContentType;
    use crate::test_utils::{ASCII_COMPATIBLE_ENCODINGS, NON_ASCII_COMPATIBLE_ENCODINGS, Output};
    use encoding_rs::{Encoding, WINDOWS_1252};
    use itertools::Itertools;
    use static_assertions::assert_impl_all;
    use std::convert::TryInto;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    // Assert that HtmlRewriter with `SendHandlerTypes` is `Send`.
    assert_impl_all!(crate::send::HtmlRewriter<'_, Box<dyn FnMut(&[u8]) + Send + 'static>>: Send);

    fn write_chunks<O: OutputSink>(
        mut rewriter: HtmlRewriter<'_, O>,
        encoding: &'static Encoding,
        chunks: &[&str],
    ) {
        for chunk in chunks {
            let (chunk, _, _) = encoding.encode(chunk);

            rewriter.write(&chunk).unwrap();
        }

        rewriter.end().unwrap();
    }

    fn rewrite_html_bytes(html: &[u8], settings: Settings<'_, '_>) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(html.len());

        let mut rewriter = HtmlRewriter::new(settings, |c: &[u8]| out.extend_from_slice(c));

        rewriter.write(html).unwrap();
        rewriter.end().unwrap();

        out
    }

    #[allow(clippy::drop_non_drop)]
    #[test]
    fn handlers_lifetime_covariance() {
        // This test checks that if you have a handler with a lifetime larger than `'a` then you can
        // use it in a place where a handler of lifetime `'a` is expected. If the code below
        // compiles, then this condition holds.

        let x = AtomicUsize::new(0);

        let el_handler_static = element!("foo", |_| Ok(()));
        let el_handler_local = element!("foo", |_| {
            x.fetch_add(1, Ordering::Relaxed);
            Ok(())
        });

        let doc_handler_static = end!(|_| Ok(()));
        let doc_handler_local = end!(|_| {
            x.fetch_add(1, Ordering::Relaxed);
            Ok(())
        });

        let settings = Settings {
            document_content_handlers: vec![doc_handler_static, doc_handler_local],
            element_content_handlers: vec![el_handler_static, el_handler_local],
            encoding: AsciiCompatibleEncoding::utf_8(),
            strict: false,
            adjust_charset_on_meta_tag: false,
            ..Settings::new()
        };
        let rewriter = HtmlRewriter::new(settings, |_: &[u8]| ());

        drop(rewriter);

        drop(x);
    }

    #[test]
    fn rewrite_html_str() {
        let res = rewrite_str::<LocalHandlerTypes>(
            "<!-- 42 --><div><!--hi--></div>",
            RewriteStrSettings {
                element_content_handlers: vec![
                    element!("div", |el| {
                        el.set_tag_name("span").unwrap();
                        Ok(())
                    }),
                    comments!("div", |c| {
                        c.set_text("hello").unwrap();
                        Ok(())
                    }),
                ],
                ..RewriteStrSettings::new()
            },
        )
        .unwrap();

        assert_eq!(res, "<!-- 42 --><span><!--hello--></span>");
    }

    #[test]
    fn rewrite_incorrect_self_closing() {
        let res = rewrite_str::<LocalHandlerTypes>(
            "<title /></title><div/></div><style /></style><script /></script>
            <br/><br><embed/><embed> <svg><a/><path/><path></path></svg>",
            RewriteStrSettings {
                element_content_handlers: vec![element!("*:not(svg)", |el| {
                    el.set_attribute("s", if el.is_self_closing() { "y" } else { "n" })?;
                    el.set_attribute("c", if el.can_have_content() { "y" } else { "n" })?;
                    el.append("…", ContentType::Text);
                    Ok(())
                })],
                ..RewriteStrSettings::new()
            },
        )
        .unwrap();

        assert_eq!(
            res,
            r#"<title s="y" c="y">…</title><div s="y" c="y">…</div><style s="y" c="y">…</style><script s="y" c="y">…</script>
            <br s="y" c="n" /><br s="n" c="n"><embed s="y" c="n" /><embed s="n" c="n"> <svg><a s="y" c="n" /><path s="y" c="n" /><path s="n" c="y">…</path></svg>"#
        );
    }

    #[test]
    fn rewrite_arbitrary_settings() {
        let res = rewrite_str("<span>Some text</span>", Settings::new()).unwrap();
        assert_eq!(res, "<span>Some text</span>");
    }

    #[test]
    fn rewrite_non_utf8() {
        let text = "前<meta charset=latin1><span>中</span><!-- 後 -->";
        let rewritten = rewrite_str(
            text,
            Settings {
                encoding: encoding_rs::BIG5.try_into().unwrap(),
                adjust_charset_on_meta_tag: true,
                ..Settings::new()
            },
        )
        .unwrap();
        assert_eq!(rewritten, text);
    }

    #[test]
    fn non_ascii_compatible_encoding() {
        for encoding in &NON_ASCII_COMPATIBLE_ENCODINGS {
            assert_eq!(AsciiCompatibleEncoding::new(encoding), None);
        }
    }

    #[test]
    fn doctype_info() {
        for &enc in &ASCII_COMPATIBLE_ENCODINGS {
            let mut doctypes = Vec::default();

            {
                let rewriter = HtmlRewriter::new(
                    Settings {
                        document_content_handlers: vec![doctype!(|d| {
                            doctypes.push((d.name(), d.public_id(), d.system_id()));
                            Ok(())
                        })],
                        // NOTE: unwrap() here is intentional; it also tests `Ascii::new`.
                        encoding: enc.try_into().unwrap(),
                        ..Settings::new()
                    },
                    |_: &[u8]| {},
                );

                write_chunks(
                    rewriter,
                    enc,
                    &[
                        "<!doctype html1>",
                        "<!-- test --><div>",
                        r#"<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN" "#,
                        r#""http://www.w3.org/TR/html4/strict.dtd">"#,
                        "</div><!DoCtYPe ",
                    ],
                );
            }

            assert_eq!(
                doctypes,
                &[
                    (Some("html1".into()), None, None),
                    (
                        Some("html".into()),
                        Some("-//W3C//DTD HTML 4.01//EN".into()),
                        Some("http://www.w3.org/TR/html4/strict.dtd".into())
                    ),
                    (None, None, None),
                ]
            );
        }
    }

    #[test]
    fn rewrite_start_tags() {
        for &enc in &ASCII_COMPATIBLE_ENCODINGS {
            let actual: String = {
                let mut output = Output::new(enc);

                let rewriter = HtmlRewriter::new(
                    Settings {
                        element_content_handlers: vec![element!("*", |el| {
                            el.set_attribute("foo", "bar").unwrap();
                            el.prepend("<test></test>", ContentType::Html);
                            Ok(())
                        })],
                        encoding: enc.try_into().unwrap(),
                        ..Settings::new()
                    },
                    |c: &[u8]| output.push(c),
                );

                write_chunks(
                    rewriter,
                    enc,
                    &[
                        "<!doctype html>\n",
                        "<html>\n",
                        "   <head></head>\n",
                        "   <body>\n",
                        "       <div>Test</div>\n",
                        "   </body>\n",
                        "</html>",
                    ],
                );

                output.into()
            };

            assert_eq!(
                actual,
                concat!(
                    "<!doctype html>\n",
                    "<html foo=\"bar\"><test></test>\n",
                    "   <head foo=\"bar\"><test></test></head>\n",
                    "   <body foo=\"bar\"><test></test>\n",
                    "       <div foo=\"bar\"><test></test>Test</div>\n",
                    "   </body>\n",
                    "</html>",
                )
            );
        }
    }

    #[test]
    fn rewrite_document_content() {
        for &enc in &ASCII_COMPATIBLE_ENCODINGS {
            let actual: String = {
                let mut output = Output::new(enc);

                let rewriter = HtmlRewriter::new(
                    Settings {
                        element_content_handlers: vec![],
                        document_content_handlers: vec![
                            doc_comments!(|c| {
                                c.set_text(&(c.text() + "1337")).unwrap();
                                Ok(())
                            }),
                            doc_text!(|c| {
                                if c.last_in_text_node() {
                                    c.after("BAZ", ContentType::Text);
                                }

                                Ok(())
                            }),
                        ],
                        encoding: enc.try_into().unwrap(),
                        ..Settings::new()
                    },
                    |c: &[u8]| output.push(c),
                );

                write_chunks(
                    rewriter,
                    enc,
                    &[
                        "<!doctype html>\n",
                        "<!-- hey -->\n",
                        "<html>\n",
                        "   <head><!-- aloha --></head>\n",
                        "   <body>\n",
                        "       <div>Test</div>\n",
                        "   </body>\n",
                        "   <!-- bonjour -->\n",
                        "</html>Pshhh",
                    ],
                );

                output.into()
            };

            assert_eq!(
                actual,
                concat!(
                    "<!doctype html>\nBAZ",
                    "<!-- hey 1337-->\nBAZ",
                    "<html>\n",
                    "   BAZ<head><!-- aloha 1337--></head>\n",
                    "   BAZ<body>\n",
                    "       BAZ<div>TestBAZ</div>\n",
                    "   BAZ</body>\n",
                    "   BAZ<!-- bonjour 1337-->\nBAZ",
                    "</html>PshhhBAZ",
                )
            );
        }
    }

    #[test]
    fn rewrite_text_types() {
        for &enc in &ASCII_COMPATIBLE_ENCODINGS {
            let actual: String = {
                let mut output = Output::new(enc);

                let rewriter = HtmlRewriter::new(
                    Settings {
                        element_content_handlers: vec![],
                        document_content_handlers: vec![doc_text!(|c| {
                            let replace = match c.text_type() {
                                TextType::PlainText => 'P',
                                TextType::RCData => 'r',
                                TextType::RawText => 'R',
                                TextType::ScriptData => 'S',
                                TextType::Data => '.',
                                TextType::CDataSection => 'C',
                            };
                            let mut replaced: String = c
                                .as_str()
                                .chars()
                                .map(|c| if c == '\n' { c } else { replace })
                                .collect();
                            if c.last_in_text_node() {
                                replaced.push(';');
                            }
                            c.set_str(replaced);

                            Ok(())
                        })],
                        encoding: enc.try_into().unwrap(),
                        ..Settings::new()
                    },
                    |c: &[u8]| output.push(c),
                );

                write_chunks(
                    rewriter,
                    enc,
                    &[
                        "\n  <!doctype html> <title>rcdata</titlenot> <!--no comment rcdata</title>",
                        "\n   <textarea>rc<x> --><!--no comment </TEXTAREA> ",
                        "\n   body <!--> 1 </> 2 <noscript>nnnn</noscript>",
                        "\n  <script>scr</script> <style>style</style>",
                        "\n  <script><!-- scr --></script> <style>/*<![CDATA[*/ style /*]]>*/</style>",
                        "\n  <svg> body <![CDATA[ cdata ]]> body",
                        "\n  <script>scr</script> <style>style</style>",
                        "\n  <script><!-- com -->s</script> <style>/*<![CDATA[*/ style /*]]>*/</style>",
                        "\n  </svg>",
                    ],
                );

                output.into()
            };

            assert_eq!(
                actual,
                "\
                \n..;<!doctype html>.;<title>rrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr;</title>\
                \n...;<textarea>rrrrrrrrrrrrrrrrrrrrrrrr;</TEXTAREA>.\
                \n........;<!-->...;</>...;<noscript>RRRR;</noscript>\
                \n..;<script>SSS;</script>.;<style>RRRRR;</style>\
                \n..;<script>SSSSSSSSSSSS;</script>.;<style>RRRRRRRRRRRRRRRRRRRRRRRRRRR;</style>\
                \n..;<svg>......;<![CDATA[CCCCCCC;]]>.....\
                \n..;<script>...;</script>.;<style>.....;</style>\
                \n..;<script><!-- com -->.;</script>.;<style>..;<![CDATA[CCCCCCCCCCC;]]>..;</style>\
                \n..;</svg>\
                "
            );
        }
    }

    #[test]
    fn handler_invocation_order() {
        let handlers_executed = Arc::new(Mutex::new(Vec::default()));

        macro_rules! create_handlers {
            ($sel:expr, $idx:expr) => {
                element!($sel, {
                    let handlers_executed = ::std::sync::Arc::clone(&handlers_executed);

                    move |_| {
                        handlers_executed.lock().unwrap().push($idx);
                        Ok(())
                    }
                })
            };
        }

        let _res = rewrite_str(
            "<div><span foo></span></div>",
            RewriteStrSettings {
                element_content_handlers: vec![
                    create_handlers!("div span", 0),
                    create_handlers!("div > span", 1),
                    create_handlers!("span", 2),
                    create_handlers!("[foo]", 3),
                    create_handlers!("div span[foo]", 4),
                ],
                ..RewriteStrSettings::new()
            },
        )
        .unwrap();

        assert_eq!(*handlers_executed.lock().unwrap(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn write_esi_tags() {
        let res = rewrite_str(
            "<span><esi:include src=a></span>",
            RewriteStrSettings {
                element_content_handlers: vec![element!("esi\\:include", |el| {
                    el.replace("?", ContentType::Text);
                    Ok(())
                })],
                enable_esi_tags: true,
                ..RewriteStrSettings::new()
            },
        )
        .unwrap();

        assert_eq!(res, "<span>?</span>");
    }

    #[test]
    fn test_rewrite_adjust_charset_on_meta_tag_attribute_charset() {
        use crate::html_content::{ContentType, TextChunk};

        let enthusiastic_text_handler = || {
            doc_text!(move |text: &mut TextChunk<'_>| {
                let new_text = text.as_str().replace('!', "!!!");
                text.replace(&new_text, ContentType::Text);
                Ok(())
            })
        };

        let html: Vec<u8> = [
            r#"<meta charset="windows-1251"><html><head></head><body>I love "#
                .as_bytes()
                .to_vec(),
            vec![0xd5, 0xec, 0xb3, 0xcb, 0xdc],
            br"!</body></html>".to_vec(),
        ]
        .into_iter()
        .concat();

        let expected: Vec<u8> = html
            .iter()
            .copied()
            .flat_map(|c| match c {
                b'!' => vec![b'!', b'!', b'!'],
                c => vec![c],
            })
            .collect();

        let transformed_no_charset_adjustment: Vec<u8> = rewrite_html_bytes(
            &html,
            Settings {
                document_content_handlers: vec![enthusiastic_text_handler()],
                ..Settings::new()
            },
        );

        // Without charset adjustment the response has to be corrupted:
        assert_ne!(transformed_no_charset_adjustment, expected);

        let transformed_charset_adjustment: Vec<u8> = rewrite_html_bytes(
            &html,
            Settings {
                document_content_handlers: vec![enthusiastic_text_handler()],
                adjust_charset_on_meta_tag: true,
                ..Settings::new()
            },
        );

        // If it adapts the charset according to the meta tag everything will be correctly
        // encoded in windows-1251:
        assert_eq!(transformed_charset_adjustment, expected);
    }

    #[test]
    fn test_charset_switch_latency() {
        let html = b"<title>\xC3\xB0</title>\xC3\xB0<meta charset=latin1 attr='\xC3\xB0'>\xF0<meta attr='\xF0'>\xF0";

        struct Sink {
            out: Vec<u8>,
            charsets: Vec<(usize, AsciiCompatibleEncoding)>,
        }

        impl OutputSink for &mut Sink {
            fn handle_chunk(&mut self, chunk: &[u8]) {
                self.out.extend_from_slice(chunk);
            }

            fn set_encoding(&mut self, enc: AsciiCompatibleEncoding) {
                self.charsets.push((self.out.len(), enc));
            }
        }

        let mut sink = Sink {
            out: Vec::with_capacity(html.len()),
            charsets: vec![],
        };

        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![element!("[attr]", |el| {
                    assert_eq!(el.get_attribute("attr").unwrap(), "ð");
                    Ok(())
                })],
                document_content_handlers: vec![doc_text!(|text| {
                    assert!(matches!(text.as_str(), "ð" | ""));
                    Ok(())
                })],
                encoding: AsciiCompatibleEncoding::utf_8(),
                adjust_charset_on_meta_tag: true,
                ..Default::default()
            },
            &mut sink,
        );

        rewriter.write(html).unwrap();
        rewriter.end().unwrap();

        assert_eq!(html, sink.out.as_slice());
        assert_eq!(
            &[
                (0, AsciiCompatibleEncoding::utf_8()),
                (50, WINDOWS_1252.try_into().unwrap())
            ],
            sink.charsets.as_slice()
        );
    }

    #[test]
    fn test_flush_before_charset_switch() {
        let html = b"<head>\xC3<meta charset=latin1>\xB0</head>";
        let rewritten = rewrite_html_bytes(
            html,
            Settings {
                document_content_handlers: vec![doc_text!(|text| {
                    assert_ne!(text.as_str(), "ð");
                    Ok(())
                })],
                encoding: AsciiCompatibleEncoding::utf_8(),
                adjust_charset_on_meta_tag: true,
                ..Default::default()
            },
        );
        assert_eq!(
            "<head>ï¿½<meta charset=latin1>°</head>",
            rewritten.iter().map(|&c| char::from(c)).collect::<String>()
        );
    }

    #[test]
    fn test_rewrite_adjust_charset_on_meta_tag_attribute_content_type() {
        use crate::html_content::{ContentType, TextChunk};

        let enthusiastic_text_handler = || {
            doc_text!(move |text: &mut TextChunk<'_>| {
                let new_text = text.as_str().replace('!', "!!!");
                text.replace(&new_text, ContentType::Text);
                Ok(())
            })
        };

        let html: Vec<u8> = [
            r#"<meta http-equiv="conTent-type" content="text/html; charset=windows-1251"><html><head>"#.as_bytes(),
            br#"<meta charset="utf-8"></head><body>I love "#, // second one should be ignored
            &[0xd5, 0xec, 0xb3, 0xcb, 0xdc],
            br"!</body></html>",
        ].concat();

        let expected: Vec<u8> = html
            .iter()
            .flat_map(|c| match c {
                b'!' => b"!!!",
                c => std::slice::from_ref(c),
            })
            .copied()
            .collect();

        let transformed_no_charset_adjustment: Vec<u8> = rewrite_html_bytes(
            &html,
            Settings {
                document_content_handlers: vec![enthusiastic_text_handler()],
                ..Settings::new()
            },
        );

        // Without charset adjustment the response has to be corrupted:
        assert_ne!(transformed_no_charset_adjustment, expected);

        let transformed_charset_adjustment: Vec<u8> = rewrite_html_bytes(
            &html,
            Settings {
                document_content_handlers: vec![enthusiastic_text_handler()],
                adjust_charset_on_meta_tag: true,
                ..Settings::new()
            },
        );

        // If it adapts the charset according to the meta tag everything will be correctly
        // encoded in windows-1251:
        assert_eq!(transformed_charset_adjustment, expected);
    }

    mod fatal_errors {
        use super::*;
        use crate::html_content::Comment;
        use crate::memory::MemoryLimitExceededError;
        use crate::rewritable_units::{Element, TextChunk};

        fn create_rewriter<O: OutputSink>(
            max_allowed_memory_usage: usize,
            output_sink: O,
        ) -> HtmlRewriter<'static, O> {
            HtmlRewriter::new(
                Settings {
                    element_content_handlers: vec![element!("*", |_| Ok(()))],
                    memory_settings: MemorySettings {
                        max_allowed_memory_usage,
                        preallocated_parsing_buffer_size: 0,
                        ..MemorySettings::new()
                    },
                    ..Settings::new()
                },
                output_sink,
            )
        }

        #[test]
        fn buffer_capacity_limit() {
            const MAX: usize = 100;

            let mut rewriter = create_rewriter(MAX, |_: &[u8]| {});

            // Use two chunks for the stream to force the usage of the buffer and
            // make sure to overflow it.
            let chunk_1 = format!("<img alt=\"{}", "l".repeat(MAX / 2));
            let chunk_2 = format!("{}\" />", "r".repeat(MAX / 2));

            rewriter.write(chunk_1.as_bytes()).unwrap();

            let write_err = rewriter.write(chunk_2.as_bytes()).unwrap_err();

            match write_err {
                RewritingError::MemoryLimitExceeded(e) => assert_eq!(e, MemoryLimitExceededError),
                _ => panic!("{}", write_err),
            }
        }

        #[test]
        #[should_panic(expected = "Attempt to use the HtmlRewriter after a fatal error.")]
        fn poisoning_after_fatal_error() {
            const MAX: usize = 10;

            let mut rewriter = create_rewriter(MAX, |_: &[u8]| {});
            let chunk = format!("<img alt=\"{}", "l".repeat(MAX));

            rewriter.write(chunk.as_bytes()).unwrap_err();
            rewriter.end().unwrap_err();
        }

        fn create_rewriter_with_graceful_bail_out<O: OutputSink>(
            max_allowed_memory_usage: usize,
            output_sink: O,
        ) -> HtmlRewriter<'static, O> {
            HtmlRewriter::new(
                Settings {
                    element_content_handlers: vec![element!("*", |_| Ok(()))],
                    memory_settings: MemorySettings {
                        max_allowed_memory_usage,
                        preallocated_parsing_buffer_size: 0,
                        graceful_bail_out_on_memory_limit_exceeded: true,
                    },
                    ..Settings::new()
                },
                output_sink,
            )
        }

        /// Exercises the bail-out path inside `Arena::append()`: with two chunks where the open
        /// tag spans both, the parser can't consume chunk 1, so it gets buffered. Chunk 2's
        /// append then exceeds the memory limit. The graceful bail-out should flush both chunks
        /// to the sink as-is, so the caller can continue the response.
        #[test]
        fn test_graceful_bail_out_in_buffer_append() {
            const MAX: usize = 100;

            let mut output = Vec::<u8>::new();
            let mut rewriter = create_rewriter_with_graceful_bail_out(MAX, |c: &[u8]| {
                output.extend_from_slice(c);
            });

            let chunk_1 = format!("<img alt=\"{}", "l".repeat(MAX / 2));
            let chunk_2 = format!("{}\" />", "r".repeat(MAX / 2));

            rewriter.write(chunk_1.as_bytes()).unwrap();

            let err = rewriter.write(chunk_2.as_bytes()).unwrap_err();

            match err {
                RewritingError::MemoryLimitExceeded(e) => assert_eq!(e, MemoryLimitExceededError),
                _ => panic!("{}", err),
            }

            let expected: Vec<u8> = [chunk_1.as_bytes(), chunk_2.as_bytes()].concat();

            assert_eq!(output, expected);
        }

        /// Exercises the bail-out path inside `Arena::init_with()`: with no buffered data, the
        /// parser can't consume a chunk that ends with an unfinished tag *name* (the tag
        /// scanner keeps `tag_start` set, so everything from there onwards is unconsumed). The
        /// unconsumed bytes are bigger than the limit, so `init_with` fails. The graceful
        /// bail-out should flush the entire chunk to the sink as-is.
        #[test]
        fn test_graceful_bail_out_in_buffer_init_with() {
            const MAX: usize = 1;

            let mut output = Vec::<u8>::new();
            // No element handlers, so we avoid allocating the selectors VM stack which would
            // fail first with such a tight limit.
            let mut rewriter = HtmlRewriter::new(
                Settings {
                    memory_settings: MemorySettings {
                        max_allowed_memory_usage: MAX,
                        preallocated_parsing_buffer_size: 0,
                        graceful_bail_out_on_memory_limit_exceeded: true,
                    },
                    ..Settings::new()
                },
                |c: &[u8]| output.extend_from_slice(c),
            );

            // Unfinished tag name: the scanner can't call `finish_tag_name()` (no space or
            // `>`), so `tag_start` stays set and the whole chunk becomes unconsumed.
            let chunk = b"<im";

            let err = rewriter.write(chunk).unwrap_err();

            match err {
                RewritingError::MemoryLimitExceeded(e) => assert_eq!(e, MemoryLimitExceededError),
                _ => panic!("{}", err),
            }

            assert_eq!(output, chunk);
        }

        /// Exercises the bail-out path inside `Parser::parse()`: the selectors VM stack push
        /// exceeds the memory limit while processing the very first start tag, so the parser
        /// returns an error mid-chunk. The graceful bail-out flushes everything from
        /// `remaining_content_start` onwards, which (since no lexeme has been consumed yet)
        /// covers the whole chunk.
        #[test]
        fn test_graceful_bail_out_in_parser() {
            // Too small for even the initial selectors VM stack allocation.
            const MAX: usize = 16;

            let mut output = Vec::<u8>::new();
            let mut rewriter = create_rewriter_with_graceful_bail_out(MAX, |c: &[u8]| {
                output.extend_from_slice(c);
            });

            let chunk = b"<div>foo</div>";

            let err = rewriter.write(chunk).unwrap_err();

            match err {
                RewritingError::MemoryLimitExceeded(e) => assert_eq!(e, MemoryLimitExceededError),
                _ => panic!("{}", err),
            }

            assert_eq!(output, chunk);
        }

        /// Verifies that transformations applied to tokens processed before the failure point
        /// are preserved, and that the rest of the input is flushed as-is. This mirrors the
        /// contract the caller relies on: the sink contains the transformed prefix and the raw
        /// suffix, and feeding the next chunk of the original response continues it correctly.
        ///
        /// Uses a document-level comment handler (no selectors VM, so we don't burn memory on
        /// the VM stack) and the buffer-append bail-out path: chunk 1 contains a complete
        /// comment that the handler transforms, plus the start of an unfinished tag that gets
        /// buffered; chunk 2 then overflows the buffer.
        #[test]
        fn test_graceful_bail_out_preserves_prefix_transformations() {
            const MAX: usize = 100;

            let mut output = Vec::<u8>::new();
            let mut rewriter = HtmlRewriter::new(
                Settings {
                    document_content_handlers: vec![doc_comments!(|c| {
                        let text = c.text();
                        c.set_text(&format!("REWRITTEN-{text}")).unwrap();
                        Ok(())
                    })],
                    memory_settings: MemorySettings {
                        max_allowed_memory_usage: MAX,
                        preallocated_parsing_buffer_size: 0,
                        graceful_bail_out_on_memory_limit_exceeded: true,
                    },
                    ..Settings::new()
                },
                |c: &[u8]| output.extend_from_slice(c),
            );

            // chunk_1: a complete comment that the handler will transform, followed by an
            // unfinished tag whose remaining bytes get buffered for the next write.
            let chunk_1 = format!("<!--hello--><img alt=\"{}", "l".repeat(50));
            // chunk_2: trying to append this to the buffer exceeds the limit.
            let chunk_2 = format!("{}\" />", "r".repeat(50));

            rewriter.write(chunk_1.as_bytes()).unwrap();

            let err = rewriter.write(chunk_2.as_bytes()).unwrap_err();

            assert!(
                matches!(err, RewritingError::MemoryLimitExceeded(_)),
                "expected MemoryLimitExceeded, got {err}",
            );

            let output_str = std::str::from_utf8(&output).unwrap();

            // The comment must have been transformed (proves the prefix kept its handler
            // changes).
            assert!(
                output_str.contains("<!--REWRITTEN-hello-->"),
                "expected transformed comment, got {output_str:?}",
            );

            // The unfinished tag's bytes must be present raw (proves the bail-out flushed the
            // buffered + new bytes the caller had handed in).
            assert!(
                output_str.contains("<img alt=\""),
                "expected raw unfinished tag bytes, got {output_str:?}",
            );

            assert!(
                output_str.ends_with("\" />"),
                "expected raw closing of the tag at the end, got {output_str:?}",
            );

            // Sanity check: the bytes from the original input (minus what the handler
            // transformed) are all present. The transformed comment grew by some bytes; the
            // rest is byte-for-byte.
            let original_input_minus_comment =
                format!("<img alt=\"{}{}\" />", "l".repeat(50), "r".repeat(50),);

            assert!(
                output_str.contains(&original_input_minus_comment),
                "expected original (raw) suffix in output, got {output_str:?}",
            );
        }

        /// Sanity check: without the opt-in flag, the existing behavior is preserved (the
        /// sink does NOT receive the unprocessed bytes after a memory error).
        #[test]
        fn test_no_graceful_bail_out_by_default() {
            const MAX: usize = 100;

            let mut output = Vec::<u8>::new();
            // `create_rewriter` uses default MemorySettings: graceful bail-out is off.
            let mut rewriter = create_rewriter(MAX, |c: &[u8]| output.extend_from_slice(c));

            let chunk_1 = format!("<img alt=\"{}", "l".repeat(MAX / 2));
            let chunk_2 = format!("{}\" />", "r".repeat(MAX / 2));

            rewriter.write(chunk_1.as_bytes()).unwrap();
            let err = rewriter.write(chunk_2.as_bytes()).unwrap_err();

            assert!(matches!(err, RewritingError::MemoryLimitExceeded(_)));
            // Sink received nothing: chunk_1 was buffered (never emitted), chunk_2 couldn't be
            // appended, and we didn't bail out gracefully.
            assert!(
                output.is_empty(),
                "without graceful bail-out the sink should be empty, got {output:?}",
            );
        }

        // --- Response reconstruction tests ---
        //
        // Each test below verifies that, after a `MemoryLimitExceeded` bail-out, the caller
        // can reconstruct the complete response by concatenating:
        //
        //   sink_output  +  unfed_remaining_bytes  ==  original_html
        //
        // The handlers used are no-ops, so serialized tokens are byte-for-byte identical to
        // the original input, and the assertion is an exact byte comparison.
        //
        // Note: CDATA sections (`<![CDATA[...]]>`) are not tested here because CDATA content
        // is emitted incrementally by the lexer (it only needs to buffer the partial `]]>`
        // closing marker, not the whole section), so it doesn't cause Arena growth.

        fn bail_out_settings(max_memory: usize) -> MemorySettings {
            MemorySettings {
                max_allowed_memory_usage: max_memory,
                preallocated_parsing_buffer_size: 0,
                graceful_bail_out_on_memory_limit_exceeded: true,
            }
        }

        /// Feeds `html` to a graceful-bail-out rewriter in `chunk_size`-byte pieces. When
        /// `MemoryLimitExceeded` fires (during `write()` or `end()`), the remaining unfed
        /// bytes are appended verbatim to the sink output, simulating what a caller would do:
        /// stop using the poisoned rewriter and pipe the rest of the response directly.
        ///
        /// Panics if no `MemoryLimitExceeded` error fires (test misconfiguration).
        fn reconstruct_response_on_oom(
            html: &[u8],
            chunk_size: usize,
            settings: Settings<'_, '_>,
        ) -> Vec<u8> {
            let mut output = Vec::<u8>::new();
            let mut rewriter = HtmlRewriter::new(settings, |c: &[u8]| output.extend_from_slice(c));

            let mut fed_bytes = 0;
            let mut hit_limit = false;

            for chunk in html.chunks(chunk_size) {
                match rewriter.write(chunk) {
                    Ok(()) => fed_bytes += chunk.len(),
                    Err(RewritingError::MemoryLimitExceeded(_)) => {
                        fed_bytes += chunk.len();
                        hit_limit = true;
                        break;
                    }
                    Err(e) => panic!("unexpected error: {e}"),
                }
            }

            if !hit_limit {
                // All writes succeeded; try `end()` which may trigger the error during final
                // buffer processing.
                if let Err(e) = rewriter.end() {
                    match e {
                        RewritingError::MemoryLimitExceeded(_) => hit_limit = true,
                        e => panic!("unexpected error: {e}"),
                    }
                }
            }

            assert!(
                hit_limit,
                "expected MemoryLimitExceeded but processing completed \
                 (memory limit too generous for this test)",
            );

            // Append bytes we never fed to the rewriter.
            output.extend_from_slice(&html[fed_bytes..]);

            output
        }

        /// Tag with a huge base64-encoded attribute value, the shape that caused
        /// INCIDENT-6638. The lexer buffers the entire tag until `>` is found; the buffer
        /// exceeds the memory limit before that.
        #[test]
        fn test_bail_out_reconstruct_huge_attribute() {
            let html = format!(
                "<p>Hello</p><img src=\"data:image/png;base64,{}\"><p>World</p>",
                "A".repeat(16384),
            );

            let reconstructed = reconstruct_response_on_oom(
                html.as_bytes(),
                512,
                Settings {
                    element_content_handlers: vec![element!("*", |_| Ok(()))],
                    memory_settings: bail_out_settings(8192),
                    ..Settings::new()
                },
            );

            assert_eq!(
                reconstructed,
                html.as_bytes(),
                "response with huge attribute must be reconstructable",
            );
        }

        /// Tag with hundreds of small attributes whose total length exceeds the memory limit.
        /// Same mechanism as the huge-attribute test (the lexer buffers the whole tag), just a
        /// different real-world shape.
        #[test]
        fn test_bail_out_reconstruct_many_attributes() {
            let attrs: String = (0..500)
                .map(|i| format!(" data-attr-{i}=\"value-{i}\""))
                .collect();
            let html = format!("<p>Hello</p><div{attrs}>inner</div><p>World</p>");

            let reconstructed = reconstruct_response_on_oom(
                html.as_bytes(),
                512,
                Settings {
                    element_content_handlers: vec![element!("*", |_| Ok(()))],
                    memory_settings: bail_out_settings(8192),
                    ..Settings::new()
                },
            );

            assert_eq!(
                reconstructed,
                html.as_bytes(),
                "response with many attributes must be reconstructable",
            );
        }

        /// Huge HTML comment (`<!-- ... -->`). The lexer buffers from `<!--` to `-->`, so a
        /// comment body larger than the limit overflows the Arena the same way a huge tag does.
        /// The comment handler puts the parser in lex mode for comments inside the outer
        /// `<div>`.
        #[test]
        fn test_bail_out_reconstruct_huge_comment() {
            let html = format!("<div>Before<!-- {} -->After</div>", "X".repeat(16384),);

            let reconstructed = reconstruct_response_on_oom(
                html.as_bytes(),
                512,
                Settings {
                    element_content_handlers: vec![comments!("div", |_| Ok(()))],
                    memory_settings: bail_out_settings(8192),
                    ..Settings::new()
                },
            );

            assert_eq!(
                reconstructed,
                html.as_bytes(),
                "response with huge comment must be reconstructable",
            );
        }

        /// Deeply nested non-void elements. Each `<div>` pushes a `StackItem` onto the
        /// selectors-VM stack; eventually the `LimitedVec` growth exceeds the memory limit.
        #[test]
        fn test_bail_out_reconstruct_deeply_nested() {
            let depth = 200;
            let open_tags: String = (0..depth).map(|_| "<div>".to_owned()).collect();
            let close_tags: String = (0..depth).map(|_| "</div>".to_owned()).collect();
            let html = format!("{open_tags}leaf{close_tags}");

            let reconstructed = reconstruct_response_on_oom(
                html.as_bytes(),
                512,
                Settings {
                    element_content_handlers: vec![element!("*", |_| Ok(()))],
                    memory_settings: bail_out_settings(4096),
                    ..Settings::new()
                },
            );

            assert_eq!(
                reconstructed,
                html.as_bytes(),
                "deeply nested response must be reconstructable",
            );
        }

        /// Many non-void start tags that are never closed: the selectors-VM stack grows
        /// without any pops, the same as deep nesting but a pattern more likely seen in
        /// broken or malicious HTML.
        #[test]
        fn test_bail_out_reconstruct_unclosed_tags() {
            let html: String = (0..200)
                .map(|i| format!("<span class=\"s{i}\">text "))
                .collect();

            let reconstructed = reconstruct_response_on_oom(
                html.as_bytes(),
                512,
                Settings {
                    element_content_handlers: vec![element!("*", |_| Ok(()))],
                    memory_settings: bail_out_settings(4096),
                    ..Settings::new()
                },
            );

            assert_eq!(
                reconstructed,
                html.as_bytes(),
                "response with unclosed tags must be reconstructable",
            );
        }

        // --- Content-handler-error bail-out tests ---
        //
        // These mirror the memory bail-out tests but with handler errors as the trigger. The
        // contract is the same: when `graceful_bail_out_on_content_handler_error = true`, the
        // sink receives every input byte the rewriter had been given before the error.

        /// An element handler that returns `Err` aborts the rewriter. With graceful bail-out
        /// enabled, the sink keeps every byte the caller had fed in, with the failing element
        /// and everything after it flushed raw (no transformation), and earlier elements
        /// transformed normally.
        #[test]
        fn test_graceful_bail_out_on_element_handler_error() {
            let html = b"<a>first</a><stop>middle</stop><b>last</b>";

            let mut output = Vec::<u8>::new();
            let mut rewriter = HtmlRewriter::new(
                Settings {
                    element_content_handlers: vec![
                        element!("a", |el| {
                            el.set_attribute("rewritten", "yes").unwrap();
                            Ok(())
                        }),
                        element!("stop", |_| Err("handler refused".into())),
                    ],
                    graceful_bail_out_on_content_handler_error: true,
                    ..Settings::new()
                },
                |c: &[u8]| output.extend_from_slice(c),
            );

            let err = rewriter.write(html).unwrap_err();

            assert!(
                matches!(err, RewritingError::ContentHandlerError(_)),
                "expected ContentHandlerError, got {err}",
            );

            // The full original bytes from `<stop>` onwards are present raw, and the `<a>` tag
            // before that has been transformed.
            let output_str = std::str::from_utf8(&output).unwrap();

            assert!(
                output_str.starts_with("<a rewritten=\"yes\">first</a>"),
                "expected transformed prefix, got {output_str:?}",
            );
            assert!(
                output_str.ends_with("<stop>middle</stop><b>last</b>"),
                "expected raw bytes from the failing tag onwards, got {output_str:?}",
            );
        }

        /// Without the opt-in flag, an element-handler error still aborts processing without
        /// flushing remaining bytes (existing behavior preserved).
        #[test]
        fn test_no_graceful_bail_out_on_content_handler_error_by_default() {
            let html = b"<a>first</a><stop>middle</stop>";

            let mut output = Vec::<u8>::new();
            let mut rewriter = HtmlRewriter::new(
                Settings {
                    element_content_handlers: vec![element!("stop", |_| Err(
                        "handler refused".into()
                    ))],
                    ..Settings::new()
                },
                |c: &[u8]| output.extend_from_slice(c),
            );

            let err = rewriter.write(html).unwrap_err();

            assert!(matches!(err, RewritingError::ContentHandlerError(_)));
            assert!(
                !output.ends_with(b"<stop>middle</stop>"),
                "without graceful bail-out the sink must NOT contain the failing tag, got {output:?}",
            );
        }

        /// A comment handler that returns `Err` is recoverable too. Comments live on the same
        /// `lexeme_consumed` path as elements, so this exercises the same restructured ordering.
        #[test]
        fn test_graceful_bail_out_on_comment_handler_error() {
            let html = b"<div>Before<!--FAIL-->After</div>";

            let mut output = Vec::<u8>::new();
            let mut rewriter = HtmlRewriter::new(
                Settings {
                    element_content_handlers: vec![comments!("div", |_| {
                        Err("comment refused".into())
                    })],
                    graceful_bail_out_on_content_handler_error: true,
                    ..Settings::new()
                },
                |c: &[u8]| output.extend_from_slice(c),
            );

            let err = rewriter.write(html).unwrap_err();
            assert!(matches!(err, RewritingError::ContentHandlerError(_)));

            // The whole document is in the sink (handler error doesn't lose bytes).
            assert_eq!(output, html);
        }

        /// A handler error from `handle_end` arrives after `flush_remaining_input` has already
        /// emitted every input byte, so the sink already has the complete document. Bail-out
        /// just propagates the error without losing anything.
        #[test]
        fn test_graceful_bail_out_on_end_handler_error() {
            let html = b"<div>content</div>";

            let mut output = Vec::<u8>::new();
            let mut rewriter = HtmlRewriter::new(
                Settings {
                    document_content_handlers: vec![end!(|_| Err("end refused".into()))],
                    graceful_bail_out_on_content_handler_error: true,
                    ..Settings::new()
                },
                |c: &[u8]| output.extend_from_slice(c),
            );

            rewriter.write(html).unwrap();

            let err = rewriter.end().unwrap_err();

            assert!(matches!(err, RewritingError::ContentHandlerError(_)));
            // All input bytes already in sink before `handle_end()` runs.
            assert_eq!(output, html);
        }

        /// Reconstruction test: when a handler in the middle of a document errors, the sink
        /// output plus any unfed bytes must equal the original document.
        #[test]
        fn test_bail_out_reconstruct_handler_error_midstream() {
            let html = b"<p>before</p><div>middle</div><span>after</span>";

            let mut output = Vec::<u8>::new();
            let mut rewriter = HtmlRewriter::new(
                Settings {
                    element_content_handlers: vec![element!("div", |_| {
                        Err("div refused".into())
                    })],
                    graceful_bail_out_on_content_handler_error: true,
                    ..Settings::new()
                },
                |c: &[u8]| output.extend_from_slice(c),
            );

            let err = rewriter.write(html).unwrap_err();
            assert!(matches!(err, RewritingError::ContentHandlerError(_)));

            assert_eq!(
                output, html,
                "response must be reconstructable byte-for-byte when handler errors midstream",
            );
        }

        /// The two bail-out flags are independent: enabling content-handler bail-out does not
        /// affect memory-limit behavior, and vice versa.
        #[test]
        fn test_bail_out_flags_independent() {
            // Memory limit error with content-handler bail-out only: should NOT bail out.
            const MAX: usize = 100;
            let mut output = Vec::<u8>::new();
            let mut rewriter = HtmlRewriter::new(
                Settings {
                    element_content_handlers: vec![element!("*", |_| Ok(()))],
                    memory_settings: MemorySettings {
                        max_allowed_memory_usage: MAX,
                        preallocated_parsing_buffer_size: 0,
                        graceful_bail_out_on_memory_limit_exceeded: false,
                    },
                    graceful_bail_out_on_content_handler_error: true,
                    ..Settings::new()
                },
                |c: &[u8]| output.extend_from_slice(c),
            );

            let chunk_1 = format!("<img alt=\"{}", "l".repeat(MAX / 2));
            let chunk_2 = format!("{}\" />", "r".repeat(MAX / 2));
            rewriter.write(chunk_1.as_bytes()).unwrap();
            let err = rewriter.write(chunk_2.as_bytes()).unwrap_err();

            assert!(matches!(err, RewritingError::MemoryLimitExceeded(_)));
            assert!(
                output.is_empty(),
                "content-handler flag must not enable memory bail-out, got {output:?}",
            );
        }

        #[test]
        fn content_handler_error_propagation() {
            fn assert_err<'h>(
                element_handlers: ElementContentHandlers<'h>,
                document_handlers: DocumentContentHandlers<'h>,
                expected_err: &'static str,
            ) {
                use std::borrow::Cow;

                let mut rewriter = HtmlRewriter::new(
                    Settings {
                        element_content_handlers: vec![(
                            Cow::Owned("*".parse().unwrap()),
                            element_handlers,
                        )],
                        document_content_handlers: vec![document_handlers],
                        ..Settings::new()
                    },
                    |_: &[u8]| {},
                );

                let chunks = [
                    "<!--doc comment--> Doc text",
                    "<div><!--el comment-->El text</div>",
                ];

                let mut err = None;

                for chunk in &chunks {
                    match rewriter.write(chunk.as_bytes()) {
                        Ok(()) => (),
                        Err(e) => {
                            err = Some(e);
                            break;
                        }
                    }
                }

                if err.is_none() {
                    match rewriter.end() {
                        Ok(()) => (),
                        Err(e) => err = Some(e),
                    }
                }

                let err = format!("{}", err.expect("Error expected"));

                assert_eq!(err, expected_err);
            }

            assert_err(
                ElementContentHandlers::default(),
                doc_comments!(|_| Err("Error in doc comment handler".into())),
                "Error in doc comment handler",
            );

            assert_err(
                ElementContentHandlers::default(),
                doc_text!(|_| Err("Error in doc text handler".into())),
                "Error in doc text handler",
            );

            assert_err(
                ElementContentHandlers::default(),
                doc_text!(|_| Err("Error in doctype handler".into())),
                "Error in doctype handler",
            );

            assert_err(
                ElementContentHandlers::default()
                    .element(|_: &mut Element<'_, '_, _>| Err("Error in element handler".into())),
                DocumentContentHandlers::default(),
                "Error in element handler",
            );

            assert_err(
                ElementContentHandlers::default()
                    .comments(|_: &mut Comment<'_>| Err("Error in element comment handler".into())),
                DocumentContentHandlers::default(),
                "Error in element comment handler",
            );

            assert_err(
                ElementContentHandlers::default()
                    .text(|_: &mut TextChunk<'_>| Err("Error in element text handler".into())),
                DocumentContentHandlers::default(),
                "Error in element text handler",
            );
        }

        #[test]
        fn attribute_source_locations() {
            let html = r#"<div class="foo" id='bar' data-x=baz>"#;
            let locations = Arc::new(Mutex::new(Vec::new()));
            let locations_clone = Arc::clone(&locations);

            rewrite_str::<LocalHandlerTypes>(
                html,
                RewriteStrSettings {
                    element_content_handlers: vec![element!("div", move |el| {
                        for attr in el.attributes() {
                            let name_loc = attr.name_source_location();
                            let value_loc = attr.value_source_location();
                            locations_clone.lock().unwrap().push((
                                attr.name(),
                                attr.value(),
                                name_loc.map(|l| l.bytes()),
                                value_loc.map(|l| l.bytes()),
                            ));
                        }
                        Ok(())
                    })],
                    ..RewriteStrSettings::new()
                },
            )
            .unwrap();

            let locs = locations.lock().unwrap();
            // class="foo"
            assert_eq!(locs[0].0, "class");
            assert_eq!(locs[0].1, "foo");
            assert_eq!(&html[locs[0].2.clone().unwrap()], "class");
            assert_eq!(&html[locs[0].3.clone().unwrap()], "foo");

            // id='bar'
            assert_eq!(locs[1].0, "id");
            assert_eq!(locs[1].1, "bar");
            assert_eq!(&html[locs[1].2.clone().unwrap()], "id");
            assert_eq!(&html[locs[1].3.clone().unwrap()], "bar");

            // data-x=baz (unquoted)
            assert_eq!(locs[2].0, "data-x");
            assert_eq!(locs[2].1, "baz");
            assert_eq!(&html[locs[2].2.clone().unwrap()], "data-x");
            assert_eq!(&html[locs[2].3.clone().unwrap()], "baz");
        }

        #[test]
        fn attribute_source_locations_none_for_programmatic_attributes() {
            rewrite_str::<LocalHandlerTypes>(
                "<div></div>",
                RewriteStrSettings {
                    element_content_handlers: vec![element!("div", |el| {
                        el.set_attribute("added", "val").unwrap();
                        for attr in el.attributes() {
                            if attr.name() == "added" {
                                assert!(
                                    attr.name_source_location().is_none(),
                                    "programmatic attribute should have no name source location",
                                );
                                assert!(
                                    attr.value_source_location().is_none(),
                                    "programmatic attribute should have no value source location",
                                );
                            }
                        }
                        Ok(())
                    })],
                    ..RewriteStrSettings::new()
                },
            )
            .unwrap();
        }
    }
}
