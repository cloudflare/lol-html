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

pub trait ElementData: Default + 'static {
    type MatchPayload: Hash + Eq;

    fn get_matched_payload_mut(&mut self) -> &mut HashSet<Self::MatchPayload>;
}

pub enum StackDirective {
    Push,
    PushIfNotSelfClosing,
    PopImmediately,
}

pub struct StackItem<'i, E: ElementData> {
    pub local_name: LocalName<'i>,
    pub element_data: E,
    pub jumps: Vec<AddressRange>,
    pub hereditary_jumps: Vec<AddressRange>,
    pub has_ancestor_with_hereditary_jumps: bool,
    pub stack_directive: StackDirective,
}

impl<'i, E: ElementData> StackItem<'i, E> {
    #[inline]
    pub fn new(local_name: LocalName<'i>) -> Self {
        StackItem {
            local_name,
            element_data: E::default(),
            jumps: Vec::default(),
            hereditary_jumps: Vec::default(),
            has_ancestor_with_hereditary_jumps: false,
            stack_directive: StackDirective::Push,
        }
    }

    #[inline]
    pub fn into_owned(self) -> StackItem<'static, E> {
        StackItem {
            local_name: self.local_name.into_owned(),
            element_data: self.element_data,
            jumps: self.jumps,
            hereditary_jumps: self.hereditary_jumps,
            has_ancestor_with_hereditary_jumps: self.has_ancestor_with_hereditary_jumps,
            stack_directive: self.stack_directive,
        }
    }
}

pub struct Stack<E: ElementData>(Vec<StackItem<'static, E>>);

impl<E: ElementData> Stack<E> {
    #[inline]
    pub fn get_stack_directive(&mut self, item: &StackItem<E>, ns: Namespace) -> StackDirective {
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

    pub fn pop_up_to(
        &mut self,
        local_name: LocalName,
        mut popped_element_data_handler: impl FnMut(E),
    ) {
        for i in (0..self.0.len()).rev() {
            if self.0[i].local_name == local_name {
                for item in self.0.drain(i..) {
                    popped_element_data_handler(item.element_data);
                }

                break;
            }
        }
    }

    #[inline]
    pub fn items(&self) -> &[StackItem<E>] {
        &self.0
    }

    #[inline]
    pub fn push_item(&mut self, mut item: StackItem<'static, E>) {
        if let Some(last) = self.0.last() {
            if last.has_ancestor_with_hereditary_jumps || !last.hereditary_jumps.is_empty() {
                item.has_ancestor_with_hereditary_jumps = true;
            }
        }

        self.0.push(item);
    }
}

impl<E: ElementData> Default for Stack<E> {
    #[inline]
    fn default() -> Self {
        Stack(Vec::default())
    }
}
