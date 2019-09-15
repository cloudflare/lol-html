mod handlers_dispatcher;
mod rewrite_controller;

#[macro_use]
mod settings;

use self::handlers_dispatcher::ContentHandlersDispatcher;
use self::rewrite_controller::*;
use crate::memory::MemoryLimitExceededError;
use crate::memory::MemoryLimiter;
use crate::parser::ParsingAmbiguityError;
use crate::selectors_vm::{self, SelectorMatchingVm};
use crate::transform_stream::*;
use encoding_rs::Encoding;
use failure::Error;
use std::fmt::{self, Debug};
use std::rc::Rc;

pub use self::settings::*;

fn try_encoding_from_str(encoding: &str) -> Result<&'static Encoding, EncodingError> {
    let encoding = Encoding::for_label_no_replacement(encoding.as_bytes())
        .ok_or(EncodingError::UnknownEncoding)?;

    if encoding.is_ascii_compatible() {
        Ok(encoding)
    } else {
        Err(EncodingError::NonAsciiCompatibleEncoding)
    }
}

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum EncodingError {
    #[fail(display = "Unknown character encoding has been provided.")]
    UnknownEncoding,
    #[fail(display = "Expected ASCII-compatible encoding.")]
    NonAsciiCompatibleEncoding,
}

#[derive(Fail, Debug)]
pub enum RewritingError {
    #[fail(display = "{}", _0)]
    MemoryLimitExceeded(MemoryLimitExceededError),
    #[fail(display = "{}", _0)]
    ParsingAmbiguity(ParsingAmbiguityError),
    #[fail(display = "{}", _0)]
    ContentHandlerError(Error),
}

pub struct HtmlRewriter<'h, O: OutputSink> {
    stream: TransformStream<HtmlRewriteController<'h>, O>,
    finished: bool,
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

impl<'h, O: OutputSink> HtmlRewriter<'h, O> {
    pub fn try_new<'s>(settings: Settings<'h, 's>, output_sink: O) -> Result<Self, EncodingError> {
        let encoding = try_encoding_from_str(settings.encoding)?;
        let mut selectors_ast = selectors_vm::Ast::default();
        let mut dispatcher = ContentHandlersDispatcher::default();

        for (selector, handlers) in settings.element_content_handlers {
            let locator = dispatcher.add_selector_associated_handlers(handlers);

            selectors_ast.add_selector(selector, locator);
        }

        for handlers in settings.document_content_handlers {
            dispatcher.add_document_content_handlers(handlers);
        }

        let memory_limiter =
            MemoryLimiter::new_shared(settings.memory_settings.max_allowed_memory_usage);

        let selector_matching_vm =
            SelectorMatchingVm::new(selectors_ast, encoding, Rc::clone(&memory_limiter));

        let controller = HtmlRewriteController::new(dispatcher, selector_matching_vm);

        let stream = TransformStream::new(TransformStreamSettings {
            transform_controller: controller,
            output_sink,
            preallocated_parsing_buffer_size: settings
                .memory_settings
                .preallocated_parsing_buffer_size,
            memory_limiter,
            encoding,
            strict: settings.strict,
        });

        Ok(HtmlRewriter {
            stream,
            finished: false,
            poisoned: false,
        })
    }

    #[inline]
    pub fn write(&mut self, data: &[u8]) -> Result<(), RewritingError> {
        assert!(
            !self.finished,
            "Data was written into the stream after it has ended."
        );

        guarded!(self, self.stream.write(data))
    }

    #[inline]
    pub fn end(&mut self) -> Result<(), RewritingError> {
        assert!(!self.finished, "Stream was ended twice.");
        self.finished = true;

        guarded!(self, self.stream.end())
    }
}

// NOTE: this opaque Debug implementation is required to make
// `.unwrap()` and `.expect()` methods available on Result
// returned by the `HtmlRewriterBuilder.build()` method.
impl<O: OutputSink> Debug for HtmlRewriter<'_, O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HtmlRewriter")
    }
}

pub fn rewrite_str<'h, 's>(
    html: &str,
    settings: RewriteStrSettings<'h, 's>,
) -> Result<String, RewritingError> {
    let mut output = vec![];

    // NOTE: never panics because encoding is always "utf-8".
    let mut rewriter = HtmlRewriter::try_new(settings.into(), |c: &[u8]| {
        output.extend_from_slice(c);
    })
    .unwrap();

    rewriter.write(html.as_bytes())?;
    rewriter.end()?;

    // NOTE: it's ok to unwrap here as we guarantee encoding validity of the output
    Ok(String::from_utf8(output).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_content::ContentType;
    use crate::test_utils::{Output, ASCII_COMPATIBLE_ENCODINGS};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn write_chunks<O: OutputSink>(
        rewriter: &mut HtmlRewriter<O>,
        encoding: &'static Encoding,
        chunks: &[&str],
    ) {
        for chunk in chunks {
            let (chunk, _, _) = encoding.encode(chunk);

            rewriter.write(&*chunk).unwrap();
        }

        rewriter.end().unwrap();
    }

    #[test]
    fn rewrite_html_str() {
        let res = rewrite_str(
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
                ..RewriteStrSettings::default()
            },
        )
        .unwrap();

        assert_eq!(res, "<!-- 42 --><span><!--hello--></span>");
    }

    #[test]
    fn unknown_encoding() {
        let err = HtmlRewriter::try_new(
            Settings {
                encoding: "hey-yo",
                ..Settings::default()
            },
            |_: &[u8]| {},
        )
        .unwrap_err();

        assert_eq!(err, EncodingError::UnknownEncoding);
    }

    #[test]
    fn non_ascii_compatible_encoding() {
        let err = HtmlRewriter::try_new(
            Settings {
                encoding: "utf-16be",
                ..Settings::default()
            },
            |_: &[u8]| {},
        )
        .unwrap_err();

        assert_eq!(err, EncodingError::NonAsciiCompatibleEncoding);
    }

    #[test]
    fn doctype_info() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let mut doctypes = Vec::default();

            {
                let mut rewriter = HtmlRewriter::try_new(
                    Settings {
                        document_content_handlers: vec![doctype!(|d| {
                            doctypes.push((d.name(), d.public_id(), d.system_id()));
                            Ok(())
                        })],
                        encoding: enc.name(),
                        ..Settings::default()
                    },
                    |_: &[u8]| {},
                )
                .unwrap();

                write_chunks(
                    &mut rewriter,
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
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let actual: String = {
                let mut output = Output::new(enc);

                let mut rewriter = HtmlRewriter::try_new(
                    Settings {
                        element_content_handlers: vec![element!("*", |el| {
                            el.set_attribute("foo", "bar").unwrap();
                            el.prepend("<test></test>", ContentType::Html);
                            Ok(())
                        })],
                        encoding: enc.name(),
                        ..Settings::default()
                    },
                    |c: &[u8]| output.push(c),
                )
                .unwrap();

                write_chunks(
                    &mut rewriter,
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
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let actual: String = {
                let mut output = Output::new(enc);

                let mut rewriter = HtmlRewriter::try_new(
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
                        encoding: enc.name(),
                        ..Settings::default()
                    },
                    |c: &[u8]| output.push(c),
                )
                .unwrap();

                write_chunks(
                    &mut rewriter,
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
    fn handler_invocation_order() {
        let handlers_executed = Rc::new(RefCell::new(Vec::default()));

        macro_rules! create_handlers {
            ($sel:expr, $idx:expr) => {
                element!($sel, {
                    let handlers_executed = Rc::clone(&handlers_executed);

                    move |_| {
                        handlers_executed.borrow_mut().push($idx);
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
                ..RewriteStrSettings::default()
            },
        )
        .unwrap();

        assert_eq!(*handlers_executed.borrow(), vec![0, 1, 2, 3, 4]);
    }

    mod fatal_errors {
        use super::*;
        use crate::errors::MemoryLimitExceededError;

        fn create_rewriter<O: OutputSink>(
            max_allowed_memory_usage: usize,
            output_sink: O,
        ) -> HtmlRewriter<'static, O> {
            HtmlRewriter::try_new(
                Settings {
                    element_content_handlers: vec![element!("*", |_| Ok(()))],
                    memory_settings: MemorySettings {
                        max_allowed_memory_usage,
                        preallocated_parsing_buffer_size: 0,
                    },
                    ..Settings::default()
                },
                output_sink,
            )
            .unwrap()
        }

        #[test]
        fn buffer_capacity_limit() {
            const MAX: usize = 100;

            let mut rewriter = create_rewriter(MAX, |_: &[u8]| {});

            // Use two chunks for the stream to force the usage of the buffer and
            // make sure to overflow it.
            let chunk_1 = format!("<img alt=\"{}", "l".repeat(MAX / 2));
            let chunk_2 = format!("{}\" />", "r".repeat(MAX / 2));
            let mem_used = chunk_1.len() + chunk_2.len();

            rewriter.write(chunk_1.as_bytes()).unwrap();

            let write_err = rewriter.write(chunk_2.as_bytes()).unwrap_err();

            match write_err {
                RewritingError::MemoryLimitExceeded(e) => assert_eq!(
                    e,
                    MemoryLimitExceededError {
                        current_usage: mem_used,
                        max: MAX
                    }
                ),
                _ => panic!(write_err),
            }
        }

        #[test]
        #[should_panic(expected = "Data was written into the stream after it has ended.")]
        fn write_after_end() {
            let mut rewriter = create_rewriter(512, |_: &[u8]| {});

            rewriter.end().unwrap();
            rewriter.write(b"foo").unwrap();
        }

        #[test]
        #[should_panic(expected = "Stream was ended twice.")]
        fn end_twice() {
            let mut rewriter = create_rewriter(512, |_: &[u8]| {});

            rewriter.end().unwrap();
            rewriter.end().unwrap();
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

        #[test]
        fn content_handler_error_propagation() {
            fn assert_err(
                element_handlers: ElementContentHandlers,
                document_handlers: DocumentContentHandlers,
                expected_err: &'static str,
            ) {
                let mut rewriter = HtmlRewriter::try_new(
                    Settings {
                        element_content_handlers: vec![(&"*".parse().unwrap(), element_handlers)],
                        document_content_handlers: vec![document_handlers],
                        ..Settings::default()
                    },
                    |_: &[u8]| {},
                )
                .unwrap();

                let chunks = [
                    "<!--doc comment--> Doc text",
                    "<div><!--el comment-->El text</div>",
                ];

                let mut err = None;

                for chunk in chunks.iter() {
                    match rewriter.write(chunk.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => {
                            err = Some(e);
                            break;
                        }
                    }
                }

                if err.is_none() {
                    match rewriter.end() {
                        Ok(_) => (),
                        Err(e) => err = Some(e),
                    }
                }

                let err = format!("{}", err.expect("Error expected"));

                assert_eq!(err, expected_err);
            }

            assert_err(
                ElementContentHandlers::default(),
                doc_comments!(|_| Err(format_err!("Error in doc comment handler"))),
                "Error in doc comment handler",
            );

            assert_err(
                ElementContentHandlers::default(),
                doc_text!(|_| Err(format_err!("Error in doc text handler"))),
                "Error in doc text handler",
            );

            assert_err(
                ElementContentHandlers::default(),
                doc_text!(|_| Err(format_err!("Error in doctype handler"))),
                "Error in doctype handler",
            );

            assert_err(
                ElementContentHandlers::default()
                    .element(|_| Err(format_err!("Error in element handler"))),
                DocumentContentHandlers::default(),
                "Error in element handler",
            );

            assert_err(
                ElementContentHandlers::default()
                    .comments(|_| Err(format_err!("Error in element comment handler"))),
                DocumentContentHandlers::default(),
                "Error in element comment handler",
            );

            assert_err(
                ElementContentHandlers::default()
                    .text(|_| Err(format_err!("Error in element text handler"))),
                DocumentContentHandlers::default(),
                "Error in element text handler",
            );
        }
    }
}
