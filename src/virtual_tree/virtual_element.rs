use super::open_element_stack::StackItem;
use crate::html::LocalName;
use std::borrow::Cow;

pub struct VirtualElement<'i> {
    local_name: Cow<'i, LocalName<'i>>,
    parent: Option<StackItem>,
}

impl<'i> VirtualElement<'i> {
    #[inline]
    pub fn new(local_name: &'i LocalName<'i>) -> Self {
        VirtualElement {
            local_name: Cow::Borrowed(local_name),
            parent: None,
        }
    }
}
