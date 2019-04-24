use crate::rewritable_units::{Comment, Doctype, Element, TextChunk, Token, TokenCaptureFlags};

pub type DoctypeHandler<'h> = Box<dyn FnMut(&mut Doctype) + 'h>;
pub type CommentHandler<'h> = Box<dyn FnMut(&mut Comment) + 'h>;
pub type TextHandler<'h> = Box<dyn FnMut(&mut TextChunk) + 'h>;
pub type ElementHandler<'h> = Box<dyn FnMut(&mut Element<'_, '_>) + 'h>;

struct HandlerVecItem<H> {
    handler: H,
    user_count: usize,
}

struct HandlerVec<H> {
    handlers: Vec<HandlerVecItem<H>>,
    user_count: usize,
}

impl<H> Default for HandlerVec<H> {
    fn default() -> Self {
        HandlerVec {
            handlers: Vec::default(),
            user_count: 0,
        }
    }
}

impl<H> HandlerVec<H> {
    #[inline]
    pub fn push(&mut self, handler: H, always_active: bool) -> usize {
        let idx = self.handlers.len();

        let item = HandlerVecItem {
            handler,
            user_count: if always_active { 1 } else { 0 },
        };

        self.user_count += item.user_count;
        self.handlers.push(item);

        idx
    }

    #[inline]
    pub fn inc_user_count(&mut self, idx: usize) {
        self.handlers[idx].user_count += 1;
        self.user_count += 1;
    }

    #[inline]
    pub fn dec_user_count(&mut self, idx: usize) {
        self.handlers[idx].user_count -= 1;
        self.user_count -= 1;
    }

    #[inline]
    pub fn has_active(&self) -> bool {
        self.user_count > 0
    }

    #[inline]
    pub fn for_each_active_handler(&mut self, mut action: impl FnMut(&mut H)) {
        for item in self.handlers.iter_mut() {
            if item.user_count > 0 {
                action(&mut item.handler);
            }
        }
    }
}

struct SelfDeactivatingHandlerVec<H> {
    handlers: Vec<HandlerVecItem<H>>,
    user_count: usize,
}

impl<H> Default for SelfDeactivatingHandlerVec<H> {
    fn default() -> Self {
        SelfDeactivatingHandlerVec {
            handlers: Vec::default(),
            user_count: 0,
        }
    }
}

impl<H> SelfDeactivatingHandlerVec<H> {
    #[inline]
    pub fn push(&mut self, handler: H) -> usize {
        let idx = self.handlers.len();

        self.handlers.push(HandlerVecItem {
            handler,
            user_count: 0,
        });

        idx
    }

    #[inline]
    pub fn use_handler(&mut self, idx: usize) {
        let handler = &mut self.handlers[idx];

        if handler.user_count == 0 {
            handler.user_count = 1;
            self.user_count += 1;
        }
    }

    #[inline]
    pub fn has_active(&self) -> bool {
        self.user_count > 0
    }

    #[inline]
    pub fn for_each_active_handler(&mut self, mut action: impl FnMut(&mut H)) {
        for item in self.handlers.iter_mut() {
            if item.user_count == 1 {
                action(&mut item.handler);
                item.user_count = 0;
            }
        }

        self.user_count = 0;
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
    element_handlers: SelfDeactivatingHandlerVec<ElementHandler<'h>>,
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
            element_handler_idx: element_handler.map(|h| self.element_handlers.push(h)),
            comment_handler_idx: comment_handler.map(|h| self.comment_handlers.push(h, false)),
            text_handler_idx: text_handler.map(|h| self.text_handlers.push(h, false)),
        }
    }

    #[inline]
    pub fn inc_element_handlers_user_count(&mut self, locator: ElementContentHandlersLocator) {
        if let Some(idx) = locator.comment_handler_idx {
            self.comment_handlers.inc_user_count(idx);
        }

        if let Some(idx) = locator.text_handler_idx {
            self.text_handlers.inc_user_count(idx);
        }

        if let Some(idx) = locator.element_handler_idx {
            self.element_handlers.use_handler(idx);
        }
    }

    #[inline]
    pub fn dec_element_handlers_user_count(&mut self, locator: ElementContentHandlersLocator) {
        if let Some(idx) = locator.comment_handler_idx {
            self.comment_handlers.dec_user_count(idx);
        }

        if let Some(idx) = locator.text_handler_idx {
            self.text_handlers.dec_user_count(idx);
        }
    }

    #[inline]
    pub fn handle_token(&mut self, token: &mut Token) {
        match token {
            Token::StartTag(start_tag) => {
                let mut element = Element::new(start_tag);

                self.element_handlers
                    .for_each_active_handler(|h| h(&mut element));
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

        if self.doctype_handlers.has_active() {
            flags |= TokenCaptureFlags::DOCTYPES;
        }

        if self.comment_handlers.has_active() {
            flags |= TokenCaptureFlags::COMMENTS;
        }

        if self.text_handlers.has_active() {
            flags |= TokenCaptureFlags::TEXT;
        }

        if self.element_handlers.has_active() {
            flags |= TokenCaptureFlags::NEXT_START_TAG;
        }

        flags
    }
}
