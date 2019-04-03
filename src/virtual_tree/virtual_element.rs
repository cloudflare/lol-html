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
}
