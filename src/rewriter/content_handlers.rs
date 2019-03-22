use crate::content::{Comment, Doctype, Element, TextChunk};

pub type DoctypeHandler<'h> = Box<dyn FnMut(&mut Doctype<'_>) + 'h>;
pub type CommentHandler<'h> = Box<dyn FnMut(&mut Comment<'_>) + 'h>;
pub type TextHandler<'h> = Box<dyn FnMut(&mut TextChunk<'_>) + 'h>;
pub type ElementHandler<'h> = Box<dyn FnMut(&mut Element<'_, '_>) + 'h>;

struct HandlerVecItem<T> {
    handler: T,
    active: bool,
}

type HandlerVec<T> = Vec<HandlerVecItem<T>>;

#[derive(Copy, Clone)]
pub struct ElementContentHandlersLocator {
    element_handler_idx: Option<usize>,
    text_chunk_handler_idx: Option<usize>,
    comment_handler_idx: Option<usize>,
}

#[derive(Default)]
pub struct ContentHandlersDispatcher<'h> {
    // NOTE: doctype handlers are always active, so we use regular vector here.
    doctype_handlers: Vec<DoctypeHandler<'h>>,
    comment_handlers: HandlerVec<CommentHandler<'h>>,
    text_chunk_handlers: HandlerVec<TextHandler<'h>>,
    element_handlers: HandlerVec<ElementHandler<'h>>,
}

impl<'h> ContentHandlersDispatcher<'h> {
    #[inline]
    fn set_element_handlers_active(
        &mut self,
        locator: ElementContentHandlersLocator,
        active: bool,
    ) {
        if let Some(idx) = locator.comment_handler_idx {
            self.comment_handlers[idx].active = active;
        }

        if let Some(idx) = locator.text_chunk_handler_idx {
            self.text_chunk_handlers[idx].active = active;
        }

        if let Some(idx) = locator.element_handler_idx {
            self.element_handlers[idx].active = active;
        }
    }
}
