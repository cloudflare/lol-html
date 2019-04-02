use super::virtual_element::VirtualElement;
use std::rc::Rc;

pub type StackItem = Rc<VirtualElement<'static>>;

pub struct OpenElementStack(Vec<StackItem>);
