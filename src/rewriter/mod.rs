mod builder;
mod content_handlers;

use self::content_handlers::*;
use crate::html::{LocalName, Namespace};
use crate::rewritable_units::{Token, TokenCaptureFlags};
use crate::selectors_vm::{MatchInfo, SelectorMatchingVm};
use crate::transform_stream::*;
use encoding_rs::Encoding;
use failure::Error;
use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::rc::Rc;

pub use self::builder::*;
pub use self::content_handlers::ElementContentHandlersLocator;

struct HtmlRewriteController<'h> {
    handlers_dispatcher: Rc<RefCell<ContentHandlersDispatcher<'h>>>,
    selector_matching_vm: SelectorMatchingVm<ElementContentHandlersLocator>,
}

impl<'h> HtmlRewriteController<'h> {
    #[inline]
    pub fn new(
        handlers_dispatcher: ContentHandlersDispatcher<'h>,
        selector_matching_vm: SelectorMatchingVm<ElementContentHandlersLocator>,
    ) -> Self {
        HtmlRewriteController {
            handlers_dispatcher: Rc::new(RefCell::new(handlers_dispatcher)),
            selector_matching_vm,
        }
    }
}

impl<'h> HtmlRewriteController<'h> {
    #[inline]
    fn create_match_handler(&self) -> impl FnMut(MatchInfo<ElementContentHandlersLocator>) + 'h {
        let handlers_dispatcher = Rc::clone(&self.handlers_dispatcher);

        move |m| {
            handlers_dispatcher
                .borrow_mut()
                .inc_element_handlers_user_count(m.payload);
        }
    }
}

impl TransformController for HtmlRewriteController<'_> {
    #[inline]
    fn initial_capture_flags(&self) -> TokenCaptureFlags {
        self.handlers_dispatcher.borrow().get_token_capture_flags()
    }

    fn handle_start_tag(
        &mut self,
        local_name: LocalName,
        ns: Namespace,
    ) -> StartTagHandlingResult<Self> {
        let mut match_handler = self.create_match_handler();

        let exec_result =
            self.selector_matching_vm
                .exec_for_start_tag(local_name, ns, &mut match_handler);

        match exec_result {
            Ok(_) => Ok(self.handlers_dispatcher.borrow().get_token_capture_flags()),
            Err(mut aux_info_req) => Err(Box::new(move |this, aux_info| {
                let mut match_handler = this.create_match_handler();

                aux_info_req(&mut this.selector_matching_vm, aux_info, &mut match_handler);

                this.handlers_dispatcher.borrow().get_token_capture_flags()
            })),
        }
    }

    fn handle_end_tag(&mut self, local_name: LocalName) -> TokenCaptureFlags {
        let handlers_dispatcher = Rc::clone(&self.handlers_dispatcher);

        self.selector_matching_vm
            .exec_for_end_tag(local_name, move |locator| {
                handlers_dispatcher
                    .borrow_mut()
                    .dec_element_handlers_user_count(locator);
            });

        self.handlers_dispatcher.borrow().get_token_capture_flags()
    }

    #[inline]
    fn handle_token(&mut self, token: &mut Token) {
        self.handlers_dispatcher.borrow_mut().handle_token(token);
    }
}

pub struct HtmlRewriter<'h, O: OutputSink>(TransformStream<HtmlRewriteController<'h>, O>);

impl<'h, O: OutputSink> HtmlRewriter<'h, O> {
    fn new(
        controller: HtmlRewriteController<'h>,
        output_sink: O,
        encoding: &'static Encoding,
    ) -> Self {
        // TODO settings
        HtmlRewriter(TransformStream::new(
            controller,
            output_sink,
            2048,
            encoding,
        ))
    }

    #[inline]
    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.0.write(data)
    }

    #[inline]
    pub fn end(&mut self) -> Result<(), Error> {
        self.0.end()
    }
}

// NOTE: this opaque Debug implementation is required to make
// `.unwrap()` and `.expect()` methods available on Result
// returned by the `HtmlRewriterBuilder.build()` method.
impl<O: OutputSink> Debug for HtmlRewriter<'_, O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HtmlRewriter")
    }
}
