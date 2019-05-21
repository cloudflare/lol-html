use super::content_handlers::*;
use super::ElementDescriptor;
use crate::rewritable_units::{Element, Token, TokenCaptureFlags};
use crate::selectors_vm::{MatchInfo, SelectorMatchingVm};

#[derive(Eq, PartialEq)]
enum ActionOnCall {
    None,
    Deactivate,
    Remove,
}

struct HandlerVecItem<H> {
    handler: H,
    user_count: usize,
    action_on_call: ActionOnCall,
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
    pub fn push(&mut self, handler: H, always_active: bool, action_on_call: ActionOnCall) -> usize {
        let idx = self.items.len();

        let item = HandlerVecItem {
            handler,
            user_count: if always_active { 1 } else { 0 },
            action_on_call,
        };

        self.user_count += item.user_count;
        self.items.push(item);

        idx
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
    pub fn call_active_handlers(&mut self, mut caller: impl FnMut(&mut H)) {
        // TODO rewrite this when drain_filter gets stable.
        for i in (0..self.items.len()).rev() {
            let item = &mut self.items[i];

            if item.user_count > 0 {
                caller(&mut item.handler);

                match item.action_on_call {
                    ActionOnCall::None => (),
                    ActionOnCall::Deactivate => {
                        self.user_count -= item.user_count;
                        item.user_count = 0;
                    }
                    ActionOnCall::Remove => {
                        self.user_count -= item.user_count;
                        self.items.remove(i);
                    }
                }
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
    pub fn has_matched_elements_with_removed_content(&self) -> bool {
        self.matched_elements_with_removed_content > 0
    }

    #[inline]
    pub fn add_document_content_handlers(
        &mut self,
        doctype_handler: Option<DoctypeHandler<'h>>,
        comment_handler: Option<CommentHandler<'h>>,
        text_handler: Option<TextHandler<'h>>,
    ) {
        // NOTE: document-level handlers are always active
        if let Some(handler) = doctype_handler {
            self.doctype_handlers
                .push(handler, true, ActionOnCall::None);
        }

        if let Some(handler) = comment_handler {
            self.comment_handlers
                .push(handler, true, ActionOnCall::None);
        }

        if let Some(handler) = text_handler {
            self.text_handlers.push(handler, true, ActionOnCall::None);
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
                self.element_handlers
                    .push(h, false, ActionOnCall::Deactivate)
            }),
            comment_handler_idx: comment_handler
                .map(|h| self.comment_handlers.push(h, false, ActionOnCall::None)),
            text_handler_idx: text_handler
                .map(|h| self.text_handlers.push(h, false, ActionOnCall::None)),
        }
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

    pub fn handle_token(
        &mut self,
        token: &mut Token,
        selector_matching_vm: &mut SelectorMatchingVm<ElementDescriptor>,
    ) {
        match token {
            Token::Doctype(doctype) => self.doctype_handlers.call_active_handlers(|h| h(doctype)),
            Token::StartTag(start_tag) => {
                if self.matched_elements_with_removed_content > 0 {
                    start_tag.mutations.remove();
                }

                let mut element = Element::new(start_tag, self.next_element_can_have_content);

                self.element_handlers
                    .call_active_handlers(|h| h(&mut element));

                if self.next_element_can_have_content {
                    if let Some(elem_desc) = selector_matching_vm.current_element_data_mut() {
                        if element.should_remove_content() {
                            elem_desc.remove_content = true;
                            self.matched_elements_with_removed_content += 1;
                        }

                        if let Some(handler) = element.into_end_tag_handler() {
                            elem_desc.end_tag_handler_idx = Some(self.end_tag_handlers.push(
                                handler,
                                false,
                                ActionOnCall::Remove,
                            ));
                        }
                    }
                }
            }
            Token::EndTag(end_tag) => {
                self.end_tag_handlers.call_active_handlers(|h| h(end_tag));
            }
            Token::TextChunk(text) => self.text_handlers.call_active_handlers(|h| h(text)),
            Token::Comment(comment) => self.comment_handlers.call_active_handlers(|h| h(comment)),
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
