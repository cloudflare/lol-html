mod builder;
mod content_handlers;
mod handlers_dispatcher;
mod rewrite_controller;

use self::handlers_dispatcher::*;
use self::rewrite_controller::*;
use crate::transform_stream::*;
use encoding_rs::Encoding;
use failure::Error;
use std::fmt::{self, Debug};

pub use self::builder::*;
pub use self::content_handlers::{EndTagHandler, SelectorHandlersLocator};

pub struct HtmlRewriter<'h, O: OutputSink>(TransformStream<HtmlRewriteController<'h>, O>);

impl<'h, O: OutputSink> HtmlRewriter<'h, O> {
    fn new(
        controller: HtmlRewriteController<'h>,
        output_sink: O,
        encoding: &'static Encoding,
    ) -> Self {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HtmlRewriter")
    }
}
