use crate::rewritable_units::{Comment, Doctype, Element, EndTag, TextChunk};

pub type DoctypeHandler<'h> = Box<dyn FnMut(&mut Doctype) + 'h>;
pub type CommentHandler<'h> = Box<dyn FnMut(&mut Comment) + 'h>;
pub type TextHandler<'h> = Box<dyn FnMut(&mut TextChunk) + 'h>;
pub type ElementHandler<'h> = Box<dyn FnMut(&mut Element) + 'h>;
pub type EndTagHandler<'h> = Box<dyn FnMut(&mut EndTag) + 'h>;

#[derive(Default)]
pub struct ElementContentHandlers<'h> {
    pub(super) element: Option<ElementHandler<'h>>,
    pub(super) comments: Option<CommentHandler<'h>>,
    pub(super) text: Option<TextHandler<'h>>,
}

impl<'h> ElementContentHandlers<'h> {
    #[inline]
    pub fn element(mut self, handler: impl FnMut(&mut Element) + 'h) -> Self {
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
    pub(super) doctype: Option<DoctypeHandler<'h>>,
    pub(super) comments: Option<CommentHandler<'h>>,
    pub(super) text: Option<TextHandler<'h>>,
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
