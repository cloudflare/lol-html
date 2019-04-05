mod builder;
mod content_handlers;

use self::content_handlers::*;
use crate::html::{LocalName, Namespace};
use crate::rewritable_units::{Token, TokenCaptureFlags};
use crate::transform_stream::*;
use encoding_rs::Encoding;
use failure::Error;
use std::fmt::{self, Debug};

pub use self::builder::*;
pub use self::content_handlers::ElementContentHandlersLocator;

#[derive(Default)]
struct HtmlRewriteController<'h> {
    handlers_dispatcher: ContentHandlersDispatcher<'h>,
    element_handler_locators: Vec<ElementContentHandlersLocator>,
}

impl TransformController for HtmlRewriteController<'_> {
    #[inline]
    fn initial_capture_flags(&self) -> TokenCaptureFlags {
        self.handlers_dispatcher.get_token_capture_flags()
    }

    fn handle_element_start(
        &mut self,
        _: LocalName<'_>,
        _: Namespace,
    ) -> ElementStartResponse<Self> {
        for &locator in &self.element_handler_locators {
            self.handlers_dispatcher
                .set_element_handlers_active(locator, true);
        }

        ElementStartResponse::CaptureFlags(self.handlers_dispatcher.get_token_capture_flags())
    }

    fn handle_element_end(&mut self, _: LocalName<'_>) -> TokenCaptureFlags {
        self.handlers_dispatcher.get_token_capture_flags()
    }

    #[inline]
    fn handle_token(&mut self, token: &mut Token<'_>) {
        self.handlers_dispatcher.handle_token(token);
    }
}

pub struct HtmlRewriter<'h, O: OutputSink>(TransformStream<HtmlRewriteController<'h>, O>);

impl<'h, O: OutputSink> HtmlRewriter<'h, O> {
    fn new(
        controller: HtmlRewriteController<'h>,
        output_sink: O,
        encoding: &'static Encoding,
    ) -> Self {
        // TODO settings
        HtmlRewriter(TransformStream::new(
            controller,
            output_sink,
            2048,
            encoding,
        ))
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HtmlRewriter")
    }
}
