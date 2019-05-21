use super::content_handlers::*;
use super::ElementDescriptor;
use crate::rewritable_units::{Element, Token, TokenCaptureFlags};
use crate::selectors_vm::{MatchInfo, SelectorMatchingVm};
use std::rc::Rc;

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
    pub fn push(&mut self, handler: H, always_active: bool, action_on_call: ActionOnCall) {
        let item = HandlerVecItem {
            handler,
            user_count: if always_active { 1 } else { 0 },
            action_on_call,
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

impl<'h> From<&ContentHandlers<'h>> for ContentHandlersDispatcher<'h> {
    fn from(handlers: &ContentHandlers<'h>) -> Self {
        let mut dispatcher = ContentHandlersDispatcher::default();

        // NOTE: document-level handlers are always active
        for handler in &handlers.doctype {
            dispatcher
                .doctype_handlers
                .push(Rc::clone(handler), true, ActionOnCall::None);
        }

        for handler in &handlers.element {
            dispatcher
                .element_handlers
                .push(Rc::clone(handler), false, ActionOnCall::Deactivate)
        }

        // NOTE: for text and comment first push selector-associated handlers
        // to preserve their original indices in selector handler locators.
        for handler in &handlers.comment {
            dispatcher
                .comment_handlers
                .push(Rc::clone(handler), false, ActionOnCall::None);
        }

        for handler in &handlers.document_comments {
            dispatcher
                .comment_handlers
                .push(Rc::clone(handler), true, ActionOnCall::None);
        }

        for handler in &handlers.text {
            dispatcher
                .text_handlers
                .push(Rc::clone(handler), false, ActionOnCall::None);
        }

        for handler in &handlers.document_text {
            dispatcher
                .text_handlers
                .push(Rc::clone(handler), true, ActionOnCall::None);
        }

        dispatcher
    }
}

impl<'h> ContentHandlersDispatcher<'h> {
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

    pub fn handle_token(
        &mut self,
        token: &mut Token,
        selector_matching_vm: &mut SelectorMatchingVm<ElementDescriptor>,
    ) {
        macro_rules! call_handlers {
            ($handlers:expr, $arg:expr) => {
                $handlers.call_active_handlers(|h| {
                    // NOTE: if we have handler already borrowed that means that
                    // `.write()` or `.end()` of the current rewriter were invoked
                    // from one of the handlers of the other rewriter that has been
                    // constructed using the same builder. We have nothing better to do
                    // than just panic.
                    let handler = &mut *h.try_borrow_mut().expect(concat!(
                        ".write() or .end() method of the rewriter has been called from one of the",
                        " content handlers of the rewriter constructed using the same builder. ",
                        "This behaviour is forbidden due to concurrency ambiguities."
                    ));

                    handler($arg);
                });
            };
        }

        match token {
            Token::Doctype(doctype) => call_handlers!(self.doctype_handlers, doctype),

            Token::StartTag(start_tag) => {
                if self.matched_elements_with_removed_content > 0 {
                    start_tag.mutations.remove();
                }

                let mut element = Element::new(start_tag, self.next_element_can_have_content);

                call_handlers!(self.element_handlers, &mut element);

                if self.next_element_can_have_content {
                    if let Some(elem_desc) = selector_matching_vm.current_element_data_mut() {
                        if element.should_remove_content() {
                            elem_desc.remove_content = true;
                            self.matched_elements_with_removed_content += 1;
                        }

                        if let Some(handler) = element.into_end_tag_handler() {
                            elem_desc.end_tag_handler_idx = Some(self.end_tag_handlers.len());

                            self.end_tag_handlers
                                .push(handler, false, ActionOnCall::Remove);
                        }
                    }
                }
            }
            Token::EndTag(end_tag) => call_handlers!(self.end_tag_handlers, end_tag),
            Token::TextChunk(text) => call_handlers!(self.text_handlers, text),
            Token::Comment(comment) => call_handlers!(self.comment_handlers, comment),
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
