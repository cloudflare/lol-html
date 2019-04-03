use super::virtual_element::VirtualElement;
use crate::html::LocalName;
use std::rc::Rc;

pub type StackItem = Rc<VirtualElement<'static>>;

pub struct OpenElementStack(Vec<StackItem>);

impl OpenElementStack {
    //pub fn try_push<'i>(element: VirtualElement<'i>) {}
}
