mod content_handlers;
mod handlers_dispatcher;
mod rewrite_controller;

use self::handlers_dispatcher::ContentHandlersDispatcher;
use self::rewrite_controller::*;
use crate::memory::MemoryLimiter;
use crate::selectors_vm::{self, Selector, SelectorMatchingVm};
use crate::transform_stream::*;
use encoding_rs::Encoding;
use failure::Error;
use std::convert::TryFrom;
use std::fmt::{self, Debug};
use std::rc::Rc;

pub use self::content_handlers::*;

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

// NOTE: exposed in C API as well, thus repr(C).
#[repr(C)]
pub struct MemorySettings {
    preallocated_parsing_buffer_size: usize,
    max_allowed_memory_usage: usize,
}

impl Default for MemorySettings {
    #[inline]
    fn default() -> Self {
        MemorySettings {
            preallocated_parsing_buffer_size: 1024,
            max_allowed_memory_usage: std::usize::MAX,
        }
    }
}

pub struct Settings<'h, 's, O: OutputSink> {
    pub element_content_handlers: Vec<(&'s Selector, ElementContentHandlers<'h>)>,
    pub document_content_handlers: Vec<DocumentContentHandlers<'h>>,
    pub encoding: &'s str,
    pub memory_settings: MemorySettings,
    pub output_sink: O,
    pub strict: bool,
}

pub struct HtmlRewriter<'h, O: OutputSink> {
    stream: TransformStream<HtmlRewriteController<'h>, O>,
    finished: bool,
    poisoned: bool,
}

impl<'h, 's, O: OutputSink> TryFrom<Settings<'h, 's, O>> for HtmlRewriter<'h, O> {
    type Error = EncodingError;

    fn try_from(settings: Settings<'h, 's, O>) -> Result<Self, Self::Error> {
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
            output_sink: settings.output_sink,
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
    #[inline]
    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        assert!(
            !self.finished,
            "Data was written into the stream after it has ended."
        );

        guarded!(self, self.stream.write(data))
    }

    #[inline]
    pub fn end(&mut self) -> Result<(), Error> {
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
    fn unknown_encoding() {
        let err = HtmlRewriter::try_from(Settings {
            element_content_handlers: vec![],
            document_content_handlers: vec![],
            encoding: "hey-yo",
            memory_settings: MemorySettings::default(),
            output_sink: |_: &[u8]| {},
            strict: true,
        })
        .unwrap_err();

        assert_eq!(err, EncodingError::UnknownEncoding);
    }

    #[test]
    fn non_ascii_compatible_encoding() {
        let err = HtmlRewriter::try_from(Settings {
            element_content_handlers: vec![],
            document_content_handlers: vec![],
            encoding: "utf-16be",
            memory_settings: MemorySettings::default(),
            output_sink: |_: &[u8]| {},
            strict: true,
        })
        .unwrap_err();

        assert_eq!(err, EncodingError::NonAsciiCompatibleEncoding);
    }

    #[test]
    fn doctype_info() {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let mut doctypes = Vec::default();

            {
                let mut rewriter = HtmlRewriter::try_from(Settings {
                    element_content_handlers: vec![],
                    document_content_handlers: vec![DocumentContentHandlers::default().doctype(
                        |d| {
                            doctypes.push((d.name(), d.public_id(), d.system_id()));
                            Ok(())
                        },
                    )],
                    encoding: enc.name(),
                    memory_settings: MemorySettings::default(),
                    output_sink: |_: &[u8]| {},
                    strict: true,
                })
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

                let mut rewriter = HtmlRewriter::try_from(Settings {
                    element_content_handlers: vec![(
                        &"*".parse().unwrap(),
                        ElementContentHandlers::default().element(|el| {
                            el.set_attribute("foo", "bar").unwrap();
                            el.prepend("<test></test>", ContentType::Html);
                            Ok(())
                        }),
                    )],
                    document_content_handlers: vec![],
                    encoding: enc.name(),
                    memory_settings: MemorySettings::default(),
                    output_sink: |c: &[u8]| output.push(c),
                    strict: true,
                })
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

                let mut rewriter = HtmlRewriter::try_from(Settings {
                    element_content_handlers: vec![],
                    document_content_handlers: vec![DocumentContentHandlers::default()
                        .comments(|c| {
                            c.set_text(&(c.text() + "1337")).unwrap();
                            Ok(())
                        })
                        .text(|c| {
                            if c.last_in_text_node() {
                                c.after("BAZ", ContentType::Text);
                            }

                            Ok(())
                        })],
                    encoding: enc.name(),
                    memory_settings: MemorySettings::default(),
                    output_sink: |c: &[u8]| output.push(c),
                    strict: true,
                })
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
                (
                    &$sel.parse().unwrap(),
                    ElementContentHandlers::default().element({
                        let handlers_executed = Rc::clone(&handlers_executed);

                        move |_| {
                            handlers_executed.borrow_mut().push($idx);
                            Ok(())
                        }
                    }),
                )
            };
        }

        let mut rewriter = HtmlRewriter::try_from(Settings {
            element_content_handlers: vec![
                create_handlers!("div span", 0),
                create_handlers!("div > span", 1),
                create_handlers!("span", 2),
                create_handlers!("[foo]", 3),
                create_handlers!("div span[foo]", 4),
            ],
            document_content_handlers: vec![],
            encoding: "utf-8",
            memory_settings: MemorySettings::default(),
            output_sink: |_: &[u8]| {},
            strict: true,
        })
        .unwrap();

        rewriter.write(b"<div><span foo></span></div>").unwrap();
        rewriter.end().unwrap();

        assert_eq!(*handlers_executed.borrow(), vec![0, 1, 2, 3, 4]);
    }

    mod fatal_errors {
        use super::*;
        use crate::errors::MemoryLimitExceededError;

        fn create_rewriter<O: OutputSink>(
            max_allowed_memory_usage: usize,
            output_sink: O,
        ) -> HtmlRewriter<'static, O> {
            HtmlRewriter::try_from(Settings {
                element_content_handlers: vec![(
                    &"*".parse().unwrap(),
                    ElementContentHandlers::default().element(|_| Ok(())),
                )],
                document_content_handlers: vec![],
                encoding: "utf-8",
                memory_settings: MemorySettings {
                    max_allowed_memory_usage,
                    preallocated_parsing_buffer_size: 0,
                },
                output_sink,
                strict: true,
            })
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

            let buffer_capacity_err = write_err
                .find_root_cause()
                .downcast_ref::<MemoryLimitExceededError>()
                .unwrap();

            assert_eq!(
                *buffer_capacity_err,
                MemoryLimitExceededError {
                    current_usage: mem_used,
                    max: MAX
                }
            );
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
                let mut rewriter = HtmlRewriter::try_from(Settings {
                    element_content_handlers: vec![(&"*".parse().unwrap(), element_handlers)],
                    document_content_handlers: vec![document_handlers],
                    encoding: "utf-8",
                    memory_settings: MemorySettings::default(),
                    output_sink: |_: &[u8]| {},
                    strict: true,
                })
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
                DocumentContentHandlers::default()
                    .comments(|_| Err(format_err!("Error in doc comment handler"))),
                "Error in doc comment handler",
            );

            assert_err(
                ElementContentHandlers::default(),
                DocumentContentHandlers::default()
                    .text(|_| Err(format_err!("Error in doc text handler"))),
                "Error in doc text handler",
            );

            assert_err(
                ElementContentHandlers::default(),
                DocumentContentHandlers::default()
                    .text(|_| Err(format_err!("Error in doctype handler"))),
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
