use super::*;
use crate::content::{Comment, Doctype, Element, TextChunk};
use crate::transform_stream::*;
use encoding_rs::Encoding;

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum SelectorError {
    #[fail(display = "The selector is unsupported.")]
    UnsupportedSelector,
    #[fail(display = "Invalid CSS selector.")]
    InvalidSelector,
}

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
    pub fn comments(mut self, handler: impl FnMut(&mut Comment<'_>) + 'h) -> Self {
        self.comments = Some(Box::new(handler));

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
    comments: Option<CommentHandler<'h>>,
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
        self.comments = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn text(mut self, handler: impl FnMut(&mut TextChunk<'_>) + 'h) -> Self {
        self.text = Some(Box::new(handler));

        self
    }
}

#[derive(Default)]
pub struct HtmlRewriterBuilder<'h>(HtmlRewriteController<'h>);

impl<'h> HtmlRewriterBuilder<'h> {
    pub fn on_document(mut self, handlers: DocumentContentHandlers<'h>) -> Self {
        if let Some(handler) = handlers.doctype {
            self.0.doctype_handlers.push(handler);
            self.0.document_level_content_settings |=
                DocumentLevelContentSettings::CAPTURE_DOCTYPES;
        }

        if let Some(handler) = handlers.comments {
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

    pub fn on(
        mut self,
        _selector: &str,
        handlers: ElementContentHandlers<'h>,
    ) -> Result<Self, SelectorError> {
        if let Some(handler) = handlers.element {
            self.0.element_handlers.push(handler);
        }

        if let Some(handler) = handlers.comments {
            self.0.comment_handlers.push(handler);
        }

        if let Some(handler) = handlers.text {
            self.0.text_handlers.push(handler);
        }

        Ok(self)
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
            Ok(HtmlRewriter::new(self.0, output_sink, encoding))
        } else {
            Err(EncodingError::NonAsciiCompatibleEncoding)
        }
    }
}
