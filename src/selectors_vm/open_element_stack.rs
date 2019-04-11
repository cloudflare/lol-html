use crate::html::{LocalName, Namespace, Tag};
use crate::rewriter::ElementContentHandlersLocator;

pub type SelfClosingFlagRequest<'i> = Box<dyn FnOnce(&mut OpenElementStack, bool) -> bool + 'i>;

#[inline]
fn is_void_element(local_name: &LocalName<'_>) -> bool {
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

struct StackItem {
    local_name: LocalName<'static>,
    handlers_locators: Vec<ElementContentHandlersLocator>,
}

#[derive(Default)]
pub struct OpenElementStack(Vec<StackItem>);

impl OpenElementStack {
    #[inline]
    fn push(
        &mut self,
        local_name: LocalName<'_>,
        handlers_locators: Vec<ElementContentHandlersLocator>,
    ) {
        self.0.push(StackItem {
            local_name: local_name.into_owned(),
            handlers_locators,
        })
    }

    pub fn try_push<'i>(
        &mut self,
        local_name: LocalName<'i>,
        ns: Namespace,
        handlers_locators: Vec<ElementContentHandlersLocator>,
    ) -> Result<bool, SelfClosingFlagRequest<'i>> {
        if ns == Namespace::Html {
            if is_void_element(&local_name) {
                Ok(false)
            } else {
                self.push(local_name, handlers_locators);
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
                    this.push(local_name, handlers_locators);
                    true
                }
            }))
        }
    }

    pub fn pop_up_to(
        &mut self,
        local_name: LocalName,
        mut popped_locators_handler: impl FnMut(Vec<ElementContentHandlersLocator>),
    ) {
        for i in self.0.len() - 1..=0 {
            if self.0[i].local_name == local_name {
                for _ in i..self.0.len() {
                    popped_locators_handler(
                        self.0
                            .pop()
                            .expect("Element should be on the stack at this point")
                            .handlers_locators,
                    );
                }

                break;
            }
        }
    }
}
