mod builder;

use crate::content::{Comment, Doctype, Element, TextChunk, Token};
use crate::parser::TagNameInfo;
use crate::transform_stream::*;
use encoding_rs::Encoding;
use failure::Error;
use std::fmt::{self, Debug};

pub use self::builder::*;

type DoctypeHandler<'h> = Box<dyn FnMut(&mut Doctype<'_>) + 'h>;
type CommentHandler<'h> = Box<dyn FnMut(&mut Comment<'_>) + 'h>;
type TextHandler<'h> = Box<dyn FnMut(&mut TextChunk<'_>) + 'h>;
type ElementHandler<'h> = Box<dyn FnMut(&mut Element<'_, '_>) + 'h>;

#[derive(Default)]
struct HtmlRewriteController<'h> {
    document_level_content_settings: DocumentLevelContentSettings,
    doctype_handlers: Vec<DoctypeHandler<'h>>,
    element_handlers: Vec<ElementHandler<'h>>,
    comment_handlers: Vec<CommentHandler<'h>>,
    text_handlers: Vec<TextHandler<'h>>,
}

impl TransformController for HtmlRewriteController<'_> {
    #[inline]
    fn document_level_content_settings(&self) -> DocumentLevelContentSettings {
        self.document_level_content_settings
    }

    fn handle_element_start(&mut self, _: &TagNameInfo<'_>) -> ElementStartResponse<Self> {
        let mut settings = ContentSettingsOnElementStart::empty();

        if !self.element_handlers.is_empty() {
            settings |= ContentSettingsOnElementStart::CAPTURE_START_TAG_FOR_ELEMENT;
        }

        if !self.text_handlers.is_empty() {
            settings |= ContentSettingsOnElementStart::CAPTURE_TEXT;
        }

        if !self.comment_handlers.is_empty() {
            settings |= ContentSettingsOnElementStart::CAPTURE_COMMENTS;
        }

        ElementStartResponse::ContentSettings(settings)
    }

    fn handle_element_end(&mut self, _: &TagNameInfo<'_>) -> ContentSettingsOnElementEnd {
        let mut settings = ContentSettingsOnElementEnd::empty();

        if !self.text_handlers.is_empty() {
            settings |= ContentSettingsOnElementEnd::CAPTURE_TEXT;
        }

        if !self.comment_handlers.is_empty() {
            settings |= ContentSettingsOnElementEnd::CAPTURE_COMMENTS;
        }

        settings
    }

    fn handle_token(&mut self, token: &mut Token<'_>) {
        match token {
            Token::StartTag(start_tag) => {
                let mut element = Element::new(start_tag);

                self.element_handlers
                    .iter_mut()
                    .for_each(|h| h(&mut element));
            }
            Token::Doctype(doctype) => self.doctype_handlers.iter_mut().for_each(|h| h(doctype)),
            Token::TextChunk(text) => self.text_handlers.iter_mut().for_each(|h| h(text)),
            Token::Comment(comment) => self.comment_handlers.iter_mut().for_each(|h| h(comment)),
            _ => (),
        }
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
