use super::*;
use crate::rewritable_units::{Comment, Doctype, Element, TextChunk};
use crate::selectors_vm::{self, SelectorError, SelectorMatchingVm};
use crate::transform_stream::*;
use encoding_rs::Encoding;

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum EncodingError {
    #[fail(display = "Unknown character encoding has been provided.")]
    UnknownEncoding,
    #[fail(display = "Expected ASCII-compatible encoding.")]
    NonAsciiCompatibleEncoding,
}

#[derive(Default)]
pub struct ElementContentHandlers<'h> {
    element: Option<ElementHandler<'h>>,
    comments: Option<CommentHandler<'h>>,
    text: Option<TextHandler<'h>>,
}

impl<'h> ElementContentHandlers<'h> {
    #[inline]
    pub fn element(mut self, handler: impl FnMut(&mut Element<'_, '_>) + 'h) -> Self {
        self.element = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn comments(mut self, handler: impl FnMut(&mut Comment) + 'h) -> Self {
        self.comments = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn text(mut self, handler: impl FnMut(&mut TextChunk) + 'h) -> Self {
        self.text = Some(Box::new(handler));

        self
    }
}

#[derive(Default)]
pub struct DocumentContentHandlers<'h> {
    doctype: Option<DoctypeHandler<'h>>,
    comments: Option<CommentHandler<'h>>,
    text: Option<TextHandler<'h>>,
}

impl<'h> DocumentContentHandlers<'h> {
    #[inline]
    pub fn doctype(mut self, handler: impl FnMut(&mut Doctype) + 'h) -> Self {
        self.doctype = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn comments(mut self, handler: impl FnMut(&mut Comment) + 'h) -> Self {
        self.comments = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn text(mut self, handler: impl FnMut(&mut TextChunk) + 'h) -> Self {
        self.text = Some(Box::new(handler));

        self
    }
}

#[derive(Default)]
pub struct HtmlRewriterBuilder<'h> {
    handlers_dispatcher: ContentHandlersDispatcher<'h>,
    selectors_ast: selectors_vm::Ast<ElementContentHandlersLocator>,
}

impl<'h> HtmlRewriterBuilder<'h> {
    pub fn on_document(&mut self, handlers: DocumentContentHandlers<'h>) {
        self.handlers_dispatcher.add_document_content_handlers(
            handlers.doctype,
            handlers.comments,
            handlers.text,
        )
    }

    pub fn on(
        &mut self,
        selector: &str,
        handlers: ElementContentHandlers<'h>,
    ) -> Result<(), SelectorError> {
        let locator = self.handlers_dispatcher.add_element_content_handlers(
            handlers.element,
            handlers.comments,
            handlers.text,
        );

        self.selectors_ast.add_selector(selector, locator)
    }

    #[inline]
    pub fn build<O: OutputSink>(
        self,
        encoding: &str,
        output_sink: O,
    ) -> Result<HtmlRewriter<'h, O>, EncodingError> {
        let encoding = Encoding::for_label_no_replacement(encoding.as_bytes())
            .ok_or(EncodingError::UnknownEncoding)?;

        if encoding.is_ascii_compatible() {
            let selector_matching_vm = SelectorMatchingVm::new(self.selectors_ast, encoding);

            let controller =
                HtmlRewriteController::new(self.handlers_dispatcher, selector_matching_vm);

            Ok(HtmlRewriter::new(controller, output_sink, encoding))
        } else {
            Err(EncodingError::NonAsciiCompatibleEncoding)
        }
    }
}
