use super::content_handlers::*;
use super::handlers_dispatcher::ContentHandlersDispatcher;
use super::{HtmlRewriteController, HtmlRewriter};
use crate::rewritable_units::{Comment, Doctype, Element, TextChunk};
use crate::selectors_vm::{self, SelectorError, SelectorMatchingVm};
use crate::transform_stream::*;
use encoding_rs::Encoding;
use std::cell::RefCell;
use std::rc::Rc;

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

macro_rules! wrap_handler {
    ($handler:expr) => {
        Some(Rc::new(RefCell::new($handler)))
    };
}

impl<'h> ElementContentHandlers<'h> {
    #[inline]
    pub fn element(mut self, handler: impl FnMut(&mut Element) + 'h) -> Self {
        self.element = wrap_handler!(handler);

        self
    }

    #[inline]
    pub fn comments(mut self, handler: impl FnMut(&mut Comment) + 'h) -> Self {
        self.comments = wrap_handler!(handler);

        self
    }

    #[inline]
    pub fn text(mut self, handler: impl FnMut(&mut TextChunk) + 'h) -> Self {
        self.text = wrap_handler!(handler);

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
        self.doctype = wrap_handler!(handler);

        self
    }

    #[inline]
    pub fn comments(mut self, handler: impl FnMut(&mut Comment) + 'h) -> Self {
        self.comments = wrap_handler!(handler);

        self
    }

    #[inline]
    pub fn text(mut self, handler: impl FnMut(&mut TextChunk) + 'h) -> Self {
        self.text = wrap_handler!(handler);

        self
    }
}

#[derive(Default)]
pub struct HtmlRewriterBuilder<'h> {
    content_handlers: ContentHandlers<'h>,
    selectors_ast: selectors_vm::Ast<SelectorHandlersLocator>,
}

impl<'h> HtmlRewriterBuilder<'h> {
    pub fn on_document(&mut self, handlers: DocumentContentHandlers<'h>) {
        self.content_handlers.add_document_content_handlers(
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
        let locator = self.content_handlers.add_selector_associated_handlers(
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
            let selector_matching_vm = SelectorMatchingVm::new(&self.selectors_ast, encoding);
            let dispatcher = ContentHandlersDispatcher::from(&self.content_handlers);
            let controller = HtmlRewriteController::new(dispatcher, selector_matching_vm);

            Ok(HtmlRewriter::new(controller, output_sink, encoding))
        } else {
            Err(EncodingError::NonAsciiCompatibleEncoding)
        }
    }
}
