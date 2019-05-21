use crate::rewritable_units::{Comment, Doctype, Element, EndTag, TextChunk};
use std::cell::RefCell;
use std::rc::Rc;

pub type DoctypeHandler<'h> = Rc<RefCell<dyn FnMut(&mut Doctype) + 'h>>;
pub type CommentHandler<'h> = Rc<RefCell<dyn FnMut(&mut Comment) + 'h>>;
pub type TextHandler<'h> = Rc<RefCell<dyn FnMut(&mut TextChunk) + 'h>>;
pub type ElementHandler<'h> = Rc<RefCell<dyn FnMut(&mut Element) + 'h>>;
pub type EndTagHandler<'h> = Rc<RefCell<dyn FnMut(&mut EndTag) + 'h>>;

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct SelectorHandlersLocator {
    pub element_handler_idx: Option<usize>,
    pub comment_handler_idx: Option<usize>,
    pub text_handler_idx: Option<usize>,
}

#[derive(Default)]
pub struct ContentHandlers<'h> {
    pub doctype: Vec<DoctypeHandler<'h>>,
    pub document_text: Vec<TextHandler<'h>>,
    pub document_comments: Vec<CommentHandler<'h>>,
    pub comment: Vec<CommentHandler<'h>>,
    pub text: Vec<TextHandler<'h>>,
    pub element: Vec<ElementHandler<'h>>,
}

impl<'h> ContentHandlers<'h> {
    #[inline]
    pub fn add_document_content_handlers(
        &mut self,
        doctype_handler: Option<DoctypeHandler<'h>>,
        comment_handler: Option<CommentHandler<'h>>,
        text_handler: Option<TextHandler<'h>>,
    ) {
        if let Some(handler) = doctype_handler {
            self.doctype.push(handler);
        }

        if let Some(handler) = comment_handler {
            self.document_comments.push(handler);
        }

        if let Some(handler) = text_handler {
            self.document_text.push(handler);
        }
    }

    #[inline]
    pub fn add_selector_associated_handlers(
        &mut self,
        element_handler: Option<ElementHandler<'h>>,
        comment_handler: Option<CommentHandler<'h>>,
        text_handler: Option<TextHandler<'h>>,
    ) -> SelectorHandlersLocator {
        SelectorHandlersLocator {
            element_handler_idx: element_handler.map(|h| {
                self.element.push(h);
                self.element.len() - 1
            }),
            comment_handler_idx: comment_handler.map(|h| {
                self.comment.push(h);
                self.comment.len() - 1
            }),
            text_handler_idx: text_handler.map(|h| {
                self.text.push(h);
                self.text.len() - 1
            }),
        }
    }
}
