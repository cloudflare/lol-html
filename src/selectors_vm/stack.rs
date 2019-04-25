use super::program::AddressRange;
use crate::html::{LocalName, Namespace, Tag};

pub type SelfClosingFlagRequest<'i> = Box<dyn FnOnce(bool) -> bool + 'i>;

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

pub struct StackItem<P> {
    pub local_name: LocalName<'static>,
    pub matched_payload: Vec<P>,
    pub jumps: Vec<AddressRange>,
    pub hereditary_jumps: Vec<AddressRange>,
    pub has_ancestor_with_hereditary_jumps: bool,
}

#[derive(Default)]
pub struct Stack<P>(Vec<StackItem<P>>);

impl<P> Stack<P> {
    pub fn try_push<'i>(
        &'i mut self,
        item: StackItem<P>,
        ns: Namespace,
    ) -> Result<bool, SelfClosingFlagRequest<'i>> {
        if ns == Namespace::Html {
            Ok(if is_void_element(&item.local_name) {
                false
            } else {
                self.0.push(item);
                true
            })
        } else {
            // TODO currently we request lexeme for all foreign elements.
            // Consider adding additional event to the tag scanner which will
            // return just the self closing flag.
            Err(Box::new(move |self_closing| {
                if self_closing {
                    false
                } else {
                    self.0.push(item);
                    true
                }
            }))
        }
    }

    pub fn pop_up_to(&mut self, local_name: LocalName, mut popped_payload_handler: impl FnMut(P)) {
        for i in self.0.len() - 1..=0 {
            if self.0[i].local_name == local_name {
                for _ in i..self.0.len() {
                    self.0.pop().into_iter().for_each(|i| {
                        for payload in i.matched_payload {
                            popped_payload_handler(payload);
                        }
                    });
                }

                break;
            }
        }
    }
}
