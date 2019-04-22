use crate::rewritable_units::{Comment, Doctype, Element, TextChunk, Token, TokenCaptureFlags};

pub type DoctypeHandler<'h> = Box<dyn FnMut(&mut Doctype) + 'h>;
pub type CommentHandler<'h> = Box<dyn FnMut(&mut Comment) + 'h>;
pub type TextHandler<'h> = Box<dyn FnMut(&mut TextChunk) + 'h>;
pub type ElementHandler<'h> = Box<dyn FnMut(&mut Element<'_, '_>) + 'h>;

struct HandlerVecItem<H> {
    handler: H,
    active: bool,
}

struct HandlerVec<H> {
    handlers: Vec<HandlerVecItem<H>>,
    active_count: usize,
}

impl<H> Default for HandlerVec<H> {
    fn default() -> Self {
        HandlerVec {
            handlers: Vec::default(),
            active_count: 0,
        }
    }
}

impl<H> HandlerVec<H> {
    #[inline]
    pub fn push(&mut self, handler: H, active: bool) -> usize {
        let idx = self.handlers.len();

        self.handlers.push(HandlerVecItem { handler, active });

        if active {
            self.active_count += 1;
        }

        idx
    }

    #[inline]
    pub fn set_handler_active(&mut self, idx: usize, active: bool) {
        self.handlers[idx].active = active;

        if active {
            self.active_count += 1;
        } else {
            self.active_count -= 1;
        }
    }

    #[inline]
    pub fn if_has_active_set(&self, flags: &mut TokenCaptureFlags, value: TokenCaptureFlags) {
        if self.active_count > 0 {
            *flags |= value;
        }
    }

    #[inline]
    pub fn for_each_active_handler(&mut self, mut action: impl FnMut(&mut H)) {
        for item in self.handlers.iter_mut() {
            if item.active {
                action(&mut item.handler);
            }
        }
    }

    #[inline]
    pub fn once_for_each_active_handler(&mut self, mut action: impl FnMut(&mut H)) {
        for item in self.handlers.iter_mut() {
            if item.active {
                action(&mut item.handler);
                item.active = false;
            }
        }

        self.active_count = 0;
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct ElementContentHandlersLocator {
    element_handler_idx: Option<usize>,
    comment_handler_idx: Option<usize>,
    text_handler_idx: Option<usize>,
}

#[derive(Default)]
pub struct ContentHandlersDispatcher<'h> {
    doctype_handlers: HandlerVec<DoctypeHandler<'h>>,
    comment_handlers: HandlerVec<CommentHandler<'h>>,
    text_handlers: HandlerVec<TextHandler<'h>>,
    element_handlers: HandlerVec<ElementHandler<'h>>,
}

impl<'h> ContentHandlersDispatcher<'h> {
    #[inline]
    pub fn add_document_content_handlers(
        &mut self,
        doctype_handler: Option<DoctypeHandler<'h>>,
        comment_handler: Option<CommentHandler<'h>>,
        text_handler: Option<TextHandler<'h>>,
    ) {
        // NOTE: document-level handlers are always active
        if let Some(handler) = doctype_handler {
            self.doctype_handlers.push(handler, true);
        }

        if let Some(handler) = comment_handler {
            self.comment_handlers.push(handler, true);
        }

        if let Some(handler) = text_handler {
            self.text_handlers.push(handler, true);
        }
    }

    #[inline]
    pub fn add_element_content_handlers(
        &mut self,
        element_handler: Option<ElementHandler<'h>>,
        comment_handler: Option<CommentHandler<'h>>,
        text_handler: Option<TextHandler<'h>>,
    ) -> ElementContentHandlersLocator {
        ElementContentHandlersLocator {
            element_handler_idx: element_handler.map(|h| self.element_handlers.push(h, false)),
            comment_handler_idx: comment_handler.map(|h| self.comment_handlers.push(h, false)),
            text_handler_idx: text_handler.map(|h| self.text_handlers.push(h, false)),
        }
    }

    // TODO there might be multiple elements on the stack using the same
    // handlers (e.g. "*" selector). Use counter instead of boolean "active".
    #[inline]
    pub fn set_element_handlers_active(
        &mut self,
        locator: ElementContentHandlersLocator,
        active: bool,
    ) {
        if let Some(idx) = locator.comment_handler_idx {
            self.comment_handlers.set_handler_active(idx, active);
        }

        if let Some(idx) = locator.text_handler_idx {
            self.text_handlers.set_handler_active(idx, active);
        }

        if let Some(idx) = locator.element_handler_idx {
            self.element_handlers.set_handler_active(idx, active);
        }
    }

    #[inline]
    pub fn handle_token(&mut self, token: &mut Token) {
        match token {
            Token::StartTag(start_tag) => {
                let mut element = Element::new(start_tag);

                // NOTE: deactivate all the element handlers once we've got the element.
                self.element_handlers
                    .once_for_each_active_handler(|h| h(&mut element));
            }
            Token::Doctype(doctype) => self
                .doctype_handlers
                .for_each_active_handler(|h| h(doctype)),
            Token::TextChunk(text) => self.text_handlers.for_each_active_handler(|h| h(text)),
            Token::Comment(comment) => self
                .comment_handlers
                .for_each_active_handler(|h| h(comment)),
            _ => (),
        }
    }

    #[inline]
    pub fn get_token_capture_flags(&self) -> TokenCaptureFlags {
        let mut flags = TokenCaptureFlags::empty();

        self.doctype_handlers
            .if_has_active_set(&mut flags, TokenCaptureFlags::DOCTYPES);
        self.comment_handlers
            .if_has_active_set(&mut flags, TokenCaptureFlags::COMMENTS);
        self.text_handlers
            .if_has_active_set(&mut flags, TokenCaptureFlags::TEXT);
        self.element_handlers
            .if_has_active_set(&mut flags, TokenCaptureFlags::NEXT_START_TAG);

        flags
    }
}
