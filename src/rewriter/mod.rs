mod content_handlers;
mod handlers_dispatcher;
mod rewrite_controller;

use self::handlers_dispatcher::ContentHandlersDispatcher;
use self::rewrite_controller::*;
use crate::selectors_vm::{self, Selector, SelectorMatchingVm};
use crate::transform_stream::*;
use encoding_rs::Encoding;
use failure::Error;
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

pub struct HtmlRewriter<'h, O: OutputSink>(TransformStream<HtmlRewriteController<'h>, O>);

impl<'h, O: OutputSink> HtmlRewriter<'h, O> {
    pub fn try_new(
        element_content_handlers: Vec<(&Selector, ElementContentHandlers<'h>)>,
        document_content_handlers: Vec<DocumentContentHandlers<'h>>,
        encoding: &str,
        output_sink: O,
    ) -> Result<Self, EncodingError> {
        let encoding = try_encoding_from_str(encoding)?;
        let mut selectors_ast = selectors_vm::Ast::default();
        let mut dispatcher = ContentHandlersDispatcher::default();

        for (selector, handlers) in element_content_handlers {
            let locator = dispatcher.add_selector_associated_handlers(handlers);

            selectors_ast.add_selector(selector, locator);
        }

        for handlers in document_content_handlers {
            dispatcher.add_document_content_handlers(handlers);
        }

        let selector_matching_vm = SelectorMatchingVm::new(&selectors_ast, encoding);
        let controller = HtmlRewriteController::new(dispatcher, selector_matching_vm);
        let stream = TransformStream::new(controller, output_sink, 2048, encoding);

        Ok(HtmlRewriter(stream))
    }

    #[inline]
    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.0.write(data)
    }

    #[inline]
    pub fn end(&mut self) -> Result<(), Error> {
        self.0.end()
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
