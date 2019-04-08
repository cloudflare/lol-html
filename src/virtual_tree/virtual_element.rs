use super::open_element_stack::StackItem;
use crate::html::LocalName;
use crate::rewriter::ElementContentHandlersLocator;

pub struct VirtualElement<'i> {
    local_name: LocalName<'i>,
    parent: Option<StackItem>,
    handlers_locators: Vec<ElementContentHandlersLocator>,
}

impl<'i> VirtualElement<'i> {
    #[inline]
    pub fn new(local_name: LocalName<'i>) -> Self {
        VirtualElement {
            local_name,
            parent: None,
            handlers_locators: Vec::default(),
        }
    }

    #[inline]
    pub fn local_name(&self) -> &LocalName<'i> {
        &self.local_name
    }

    #[inline]
    pub fn set_parent(&mut self, parent: Option<StackItem>) {
        self.parent = parent;
    }

    #[inline]
    pub fn into_unbound_from_input(self) -> VirtualElement<'static> {
        VirtualElement {
            local_name: self.local_name.into_unbound_from_input(),
            parent: self.parent,
            handlers_locators: self.handlers_locators,
        }
    }
}
