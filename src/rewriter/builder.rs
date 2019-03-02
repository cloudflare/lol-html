use super::*;
use crate::content::{Comment, Doctype, Element, TextChunk};
use crate::transform_stream::*;
use encoding_rs::Encoding;

#[derive(Default)]
pub struct ElementContentHandlers<'h> {
    element: Option<ElementHandler<'h>>,
    comment: Option<CommentHandler<'h>>,
    text: Option<TextHandler<'h>>,
}

impl<'h> ElementContentHandlers<'h> {
    #[inline]
    pub fn element(mut self, handler: impl FnMut(&mut Element<'_, '_>) + 'h) -> Self {
        self.element = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn comments(mut self, handler: impl FnMut(&mut Comment<'_>) + 'h) -> Self {
        self.comment = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn text(mut self, handler: impl FnMut(&mut TextChunk<'_>) + 'h) -> Self {
        self.text = Some(Box::new(handler));

        self
    }
}

#[derive(Default)]
pub struct DocumentContentHandlers<'h> {
    doctype: Option<DoctypeHandler<'h>>,
    comment: Option<CommentHandler<'h>>,
    text: Option<TextHandler<'h>>,
}

impl<'h> DocumentContentHandlers<'h> {
    #[inline]
    pub fn doctype(mut self, handler: impl FnMut(&mut Doctype<'_>) + 'h) -> Self {
        self.doctype = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn comments(mut self, handler: impl FnMut(&mut Comment<'_>) + 'h) -> Self {
        self.comment = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn text(mut self, handler: impl FnMut(&mut TextChunk<'_>) + 'h) -> Self {
        self.text = Some(Box::new(handler));

        self
    }
}

pub struct HtmlRewriterBuilder<'h>(HtmlRewriteController<'h>);

impl<'h> HtmlRewriterBuilder<'h> {
    pub(crate) fn new() -> Self {
        HtmlRewriterBuilder(HtmlRewriteController::default())
    }

    pub fn document(mut self, handlers: DocumentContentHandlers<'h>) -> Self {
        if let Some(handler) = handlers.doctype {
            self.0.doctype_handlers.push(handler);
            self.0.document_level_content_settings |=
                DocumentLevelContentSettings::CAPTURE_DOCTYPES;
        }

        if let Some(handler) = handlers.comment {
            self.0.comment_handlers.push(handler);
            self.0.document_level_content_settings |=
                DocumentLevelContentSettings::CAPTURE_COMMENTS;
        }

        if let Some(handler) = handlers.text {
            self.0.text_handlers.push(handler);
            self.0.document_level_content_settings |= DocumentLevelContentSettings::CAPTURE_TEXT;
        }

        self
    }

    // TODO selector validation
    pub fn selector(mut self, _selector: &'h str, handlers: ElementContentHandlers<'h>) -> Self {
        if let Some(handler) = handlers.element {
            self.0.element_handlers.push(handler);
        }

        if let Some(handler) = handlers.comment {
            self.0.comment_handlers.push(handler);
        }

        if let Some(handler) = handlers.text {
            self.0.text_handlers.push(handler);
        }

        self
    }

    #[inline]
    pub fn build<O: OutputSink>(self, encoding: &str, output_sink: O) -> HtmlRewriter<'h, O> {
        // TODO validation
        let encoding = Encoding::for_label_no_replacement(encoding.as_bytes()).unwrap();

        HtmlRewriter::new(self.0, output_sink, encoding)
    }
}
