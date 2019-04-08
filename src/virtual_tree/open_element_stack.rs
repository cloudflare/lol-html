use super::virtual_element::VirtualElement;
use crate::html::{LocalName, Namespace, Tag};
use std::rc::Rc;

pub type StackItem = Rc<VirtualElement<'static>>;
pub type SelfClosingFlagRequest<'e> = Box<dyn FnOnce(&mut OpenElementStack, bool) -> bool + 'e>;

#[inline]
fn is_void_element(element: &VirtualElement<'_>) -> bool {
    let name = element.local_name();

    // NOTE: fast path for the most commonly used elements
    if tag_is_one_of!(*name, [Div, A, Span, Li, Input]) {
        return false;
    }

    tag_is_one_of!(
        *name,
        [
            Area, Base, Basefont, Bgsound, Br, Col, Embed, Hr, Img, Input, Keygen, Link, Meta,
            Param, Source, Track, Wbr
        ]
    )
}

#[derive(Default)]
pub struct OpenElementStack(Vec<StackItem>);

impl OpenElementStack {
    #[inline]
    pub fn attach_parent(&self, element: &mut VirtualElement<'_>) {
        element.set_parent(self.0.last().map(Rc::clone));
    }

    pub fn try_push_element<'e>(
        &mut self,
        element: VirtualElement<'e>,
        ns: Namespace,
    ) -> Result<bool, SelfClosingFlagRequest<'e>> {
        if ns == Namespace::Html {
            if is_void_element(&element) {
                Ok(false)
            } else {
                self.push(element);
                Ok(true)
            }
        } else {
            // TODO currently we request lexeme for all foreign elements.
            // Consider adding additional event to the tag scanner which will
            // return just the self closing flag.
            Err(Box::new(move |this, self_closing| {
                if self_closing {
                    false
                } else {
                    this.push(element);
                    true
                }
            }))
        }
    }

    pub fn pop_up_to(
        &mut self,
        local_name: LocalName,
        mut popped_element_handler: impl FnMut(Rc<VirtualElement<'_>>),
    ) {
        for i in self.0.len() - 1..=0 {
            if *self.0[i].local_name() == local_name {
                for _ in i..self.0.len() {
                    popped_element_handler(
                        self.0
                            .pop()
                            .expect("Element should be on the stack at this point"),
                    );
                }

                break;
            }
        }
    }

    #[inline]
    fn push(&mut self, element: VirtualElement<'_>) {
        self.0.push(Rc::new(element.into_unbound_from_input()));
    }
}
