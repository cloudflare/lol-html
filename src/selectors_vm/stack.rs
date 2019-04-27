use super::program::AddressRange;
use crate::html::{LocalName, Namespace, Tag};
use std::collections::HashSet;
use std::hash::Hash;

#[inline]
fn is_void_element(local_name: &LocalName) -> bool {
    // NOTE: fast path for the most commonly used elements
    if tag_is_one_of!(*local_name, [Div, A, Span, Li, Input]) {
        return false;
    }

    tag_is_one_of!(
        *local_name,
        [
            Area, Base, Basefont, Bgsound, Br, Col, Embed, Hr, Img, Input, Keygen, Link, Meta,
            Param, Source, Track, Wbr
        ]
    )
}

pub enum StackDirective {
    Push,
    PushIfNotSelfClosing,
    PopImmediately,
}

pub struct StackItem<'i, P>
where
    P: Hash + Eq,
{
    pub local_name: LocalName<'i>,
    pub matched_payload: HashSet<P>,
    pub jumps: Vec<AddressRange>,
    pub hereditary_jumps: Vec<AddressRange>,
    pub has_ancestor_with_hereditary_jumps: bool,
    pub stack_directive: StackDirective,
}

impl<'i, P> StackItem<'i, P>
where
    P: Hash + Eq,
{
    #[inline]
    pub fn new(local_name: LocalName<'i>) -> Self {
        StackItem {
            local_name,
            matched_payload: HashSet::default(),
            jumps: Vec::default(),
            hereditary_jumps: Vec::default(),
            has_ancestor_with_hereditary_jumps: false,
            stack_directive: StackDirective::Push,
        }
    }

    #[inline]
    pub fn into_owned(self) -> StackItem<'static, P> {
        StackItem {
            local_name: self.local_name.into_owned(),
            matched_payload: self.matched_payload,
            jumps: self.jumps,
            hereditary_jumps: self.hereditary_jumps,
            has_ancestor_with_hereditary_jumps: self.has_ancestor_with_hereditary_jumps,
            stack_directive: self.stack_directive,
        }
    }
}

pub struct Stack<P>(Vec<StackItem<'static, P>>)
where
    P: Hash + Eq;

impl<P> Stack<P>
where
    P: Hash + Eq,
{
    #[inline]
    pub fn get_stack_directive(&mut self, item: &StackItem<P>, ns: Namespace) -> StackDirective {
        if ns == Namespace::Html {
            if is_void_element(&item.local_name) {
                StackDirective::PopImmediately
            } else {
                StackDirective::Push
            }
        } else {
            StackDirective::PushIfNotSelfClosing
        }
    }

    pub fn pop_up_to(&mut self, local_name: LocalName, mut popped_payload_handler: impl FnMut(P)) {
        for i in (0..self.0.len()).rev() {
            if self.0[i].local_name == local_name {
                for _ in i..self.0.len() {
                    if let Some(item) = self.0.pop() {
                        for payload in item.matched_payload {
                            popped_payload_handler(payload);
                        }
                    }
                }

                break;
            }
        }
    }

    #[inline]
    pub fn items(&self) -> &[StackItem<P>] {
        &self.0
    }

    #[inline]
    pub fn push_item(&mut self, mut item: StackItem<'static, P>) {
        if let Some(last) = self.0.last() {
            if last.has_ancestor_with_hereditary_jumps || !last.hereditary_jumps.is_empty() {
                item.has_ancestor_with_hereditary_jumps = true;
            }
        }

        self.0.push(item);
    }
}

impl<P> Default for Stack<P>
where
    P: Hash + Eq,
{
    #[inline]
    fn default() -> Self {
        Stack(Vec::default())
    }
}
