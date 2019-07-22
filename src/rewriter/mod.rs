mod content_handlers;
mod handlers_dispatcher;
mod rewrite_controller;

use self::handlers_dispatcher::ContentHandlersDispatcher;
use self::rewrite_controller::*;
use crate::selectors_vm::{self, Selector, SelectorMatchingVm};
use crate::transform_stream::*;
use encoding_rs::Encoding;
use failure::Error;
use std::convert::TryFrom;
use std::fmt::{self, Debug};

pub use self::content_handlers::*;

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum EncodingError {
    #[fail(display = "Unknown character encoding has been provided.")]
    UnknownEncoding,
    #[fail(display = "Expected ASCII-compatible encoding.")]
    NonAsciiCompatibleEncoding,
}

fn try_encoding_from_str(encoding: &str) -> Result<&'static Encoding, EncodingError> {
    let encoding = Encoding::for_label_no_replacement(encoding.as_bytes())
        .ok_or(EncodingError::UnknownEncoding)?;

    if encoding.is_ascii_compatible() {
        Ok(encoding)
    } else {
        Err(EncodingError::NonAsciiCompatibleEncoding)
    }
}

pub struct Settings<'h, 's, O: OutputSink> {
    pub element_content_handlers: Vec<(&'s Selector, ElementContentHandlers<'h>)>,
    pub document_content_handlers: Vec<DocumentContentHandlers<'h>>,
    pub encoding: &'s str,
    pub buffer_capacity: usize,
    pub output_sink: O,
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

        let selector_matching_vm = SelectorMatchingVm::new(selectors_ast, encoding);
        let controller = HtmlRewriteController::new(dispatcher, selector_matching_vm);

        let stream = TransformStream::new(
            controller,
            settings.output_sink,
            settings.buffer_capacity,
            encoding,
        );

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
