use super::content_handlers::*;
use super::ElementDescriptor;
use crate::rewritable_units::{Element, StartTag, Token, TokenCaptureFlags};
use crate::selectors_vm::{MatchInfo, SelectorMatchingVm};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct SelectorHandlersLocator {
    pub element_handler_idx: Option<usize>,
    pub comment_handler_idx: Option<usize>,
    pub text_handler_idx: Option<usize>,
}

struct HandlerVecItem<H> {
    handler: H,
    user_count: usize,
}

struct HandlerVec<H> {
    items: Vec<HandlerVecItem<H>>,
    user_count: usize,
}

impl<H> Default for HandlerVec<H> {
    fn default() -> Self {
        HandlerVec {
            items: Vec::default(),
            user_count: 0,
        }
    }
}

impl<H> HandlerVec<H> {
    #[inline]
    pub fn push(&mut self, handler: H, always_active: bool) {
        let item = HandlerVecItem {
            handler,
            user_count: if always_active { 1 } else { 0 },
        };

        self.user_count += item.user_count;
        self.items.push(item);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    pub fn inc_user_count(&mut self, idx: usize) {
        self.items[idx].user_count += 1;
        self.user_count += 1;
    }

    #[inline]
    pub fn dec_user_count(&mut self, idx: usize) {
        self.items[idx].user_count -= 1;
        self.user_count -= 1;
    }

    #[inline]
    pub fn has_active(&self) -> bool {
        self.user_count > 0
    }

    #[inline]
    pub fn for_each_active(&mut self, cb: impl FnMut(&mut H)) {
        self.items
            .iter_mut()
            .filter(|h| h.user_count > 0)
            .map(|h| &mut h.handler)
            .for_each(cb);
    }

    #[inline]
    pub fn do_for_each_active_and_deactivate(&mut self, mut cb: impl FnMut(&mut H)) {
        self.user_count = self.items.iter_mut().fold(0, |user_count, item| {
            if item.user_count > 0 {
                cb(&mut item.handler);
                item.user_count = 0;
            }

            user_count + item.user_count
        });
    }

    #[inline]
    pub fn do_for_each_active_and_remove(&mut self, mut cb: impl FnMut(&mut H)) {
        for i in (0..self.items.len()).rev() {
            let item = &mut self.items[i];

            if item.user_count > 0 {
                cb(&mut item.handler);

                self.user_count -= item.user_count;
                self.items.remove(i);
            }
        }
    }
}

#[derive(Default)]
pub struct ContentHandlersDispatcher<'h> {
    doctype_handlers: HandlerVec<DoctypeHandler<'h>>,
    comment_handlers: HandlerVec<CommentHandler<'h>>,
    text_handlers: HandlerVec<TextHandler<'h>>,
    end_tag_handlers: HandlerVec<EndTagHandler<'h>>,
    element_handlers: HandlerVec<ElementHandler<'h>>,
    next_element_can_have_content: bool,
    matched_elements_with_removed_content: usize,
}

impl<'h> ContentHandlersDispatcher<'h> {
    #[inline]
    pub fn add_document_content_handlers(&mut self, handlers: DocumentContentHandlers<'h>) {
        if let Some(handler) = handlers.doctype {
            self.doctype_handlers.push(handler, true);
        }

        if let Some(handler) = handlers.comments {
            self.comment_handlers.push(handler, true);
        }

        if let Some(handler) = handlers.text {
            self.text_handlers.push(handler, true);
        }
    }

    #[inline]
    pub fn add_selector_associated_handlers(
        &mut self,
        handlers: ElementContentHandlers<'h>,
    ) -> SelectorHandlersLocator {
        SelectorHandlersLocator {
            element_handler_idx: handlers.element.map(|h| {
                self.element_handlers.push(h, false);
                self.element_handlers.len() - 1
            }),
            comment_handler_idx: handlers.comments.map(|h| {
                self.comment_handlers.push(h, false);
                self.comment_handlers.len() - 1
            }),
            text_handler_idx: handlers.text.map(|h| {
                self.text_handlers.push(h, false);
                self.text_handlers.len() - 1
            }),
        }
    }

    #[inline]
    pub fn has_matched_elements_with_removed_content(&self) -> bool {
        self.matched_elements_with_removed_content > 0
    }

    #[inline]
    pub fn start_matching(&mut self, match_info: MatchInfo<SelectorHandlersLocator>) {
        let locator = match_info.payload;

        if match_info.with_content {
            if let Some(idx) = locator.comment_handler_idx {
                self.comment_handlers.inc_user_count(idx);
            }

            if let Some(idx) = locator.text_handler_idx {
                self.text_handlers.inc_user_count(idx);
            }
        }

        if let Some(idx) = locator.element_handler_idx {
            self.element_handlers.inc_user_count(idx);
        }

        self.next_element_can_have_content = match_info.with_content;
    }

    #[inline]
    pub fn stop_matching(&mut self, elem_desc: ElementDescriptor) {
        for locator in elem_desc.matched_content_handlers {
            if let Some(idx) = locator.comment_handler_idx {
                self.comment_handlers.dec_user_count(idx);
            }

            if let Some(idx) = locator.text_handler_idx {
                self.text_handlers.dec_user_count(idx);
            }
        }

        if let Some(idx) = elem_desc.end_tag_handler_idx {
            self.end_tag_handlers.inc_user_count(idx);
        }

        if elem_desc.remove_content {
            self.matched_elements_with_removed_content -= 1;
        }
    }

    pub fn handle_start_tag(
        &mut self,
        start_tag: &mut StartTag,
        selector_matching_vm: &mut SelectorMatchingVm<ElementDescriptor>,
    ) {
        if self.matched_elements_with_removed_content > 0 {
            start_tag.mutations.remove();
        }

        let mut element = Element::new(start_tag, self.next_element_can_have_content);

        self.element_handlers
            .do_for_each_active_and_deactivate(|h| h(&mut element));

        if self.next_element_can_have_content {
            if let Some(elem_desc) = selector_matching_vm.current_element_data_mut() {
                if element.should_remove_content() {
                    elem_desc.remove_content = true;
                    self.matched_elements_with_removed_content += 1;
                }

                if let Some(handler) = element.into_end_tag_handler() {
                    elem_desc.end_tag_handler_idx = Some(self.end_tag_handlers.len());

                    self.end_tag_handlers.push(handler, false);
                }
            }
        }
    }

    pub fn handle_token(
        &mut self,
        token: &mut Token,
        selector_matching_vm: &mut SelectorMatchingVm<ElementDescriptor>,
    ) {
        match token {
            Token::Doctype(doctype) => self.doctype_handlers.for_each_active(|h| h(doctype)),
            Token::StartTag(start_tag) => self.handle_start_tag(start_tag, selector_matching_vm),
            Token::EndTag(end_tag) => self
                .end_tag_handlers
                .do_for_each_active_and_remove(|h| h(end_tag)),
            Token::TextChunk(text) => self.text_handlers.for_each_active(|h| h(text)),
            Token::Comment(comment) => self.comment_handlers.for_each_active(|h| h(comment)),
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

        if self.end_tag_handlers.has_active() {
            flags |= TokenCaptureFlags::NEXT_END_TAG;
        }

        if self.element_handlers.has_active() {
            flags |= TokenCaptureFlags::NEXT_START_TAG;
        }

        flags
    }
}
