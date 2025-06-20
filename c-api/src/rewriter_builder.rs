use super::*;
use libc::c_void;
use std::borrow::Cow;

#[repr(C)]
pub enum RewriterDirective {
    Continue,
    Stop,
}

type ElementHandler = unsafe extern "C" fn(*mut Element, *mut c_void) -> RewriterDirective;
type DoctypeHandler = unsafe extern "C" fn(*mut Doctype, *mut c_void) -> RewriterDirective;
type CommentsHandler = unsafe extern "C" fn(*mut Comment, *mut c_void) -> RewriterDirective;
type TextHandler = unsafe extern "C" fn(*mut TextChunk, *mut c_void) -> RewriterDirective;
type DocumentEndHandler = unsafe extern "C" fn(*mut DocumentEnd, *mut c_void) -> RewriterDirective;

struct ExternHandler<F> {
    func: Option<F>,
    user_data: *mut c_void,
}

impl<F> ExternHandler<F> {
    const fn new(func: Option<F>, user_data: *mut c_void) -> Self {
        Self { func, user_data }
    }
}

macro_rules! add_handler {
    ($handlers:ident, $el_ty:ident, $self:ident.$ty:ident) => {{
        if let Some(handler) = $self.$ty.func {
            // NOTE: the closure actually holds a reference to the content
            // handler object, but since we pass the object to the C side this
            // ownership information gets erased.
            // It's not a problem since handler is an extern static function that
            // will remain intact even if Rust-side builder object gets freed.
            // However, it's not a case for the user data pointer, it might become
            // invalid if content handlers object that holds it gets freed before
            // a handler invocation. Therefore, we close on a local variable instead
            // of structure field.
            let user_data = $self.$ty.user_data;

            $handlers =
                $handlers.$ty(
                    move |arg: &mut $el_ty| match unsafe { handler(arg, user_data) } {
                        RewriterDirective::Continue => Ok(()),
                        RewriterDirective::Stop => Err("The rewriter has been stopped.".into()),
                    },
                );
        }
    }};
}

pub struct ExternDocumentContentHandlers {
    doctype: ExternHandler<DoctypeHandler>,
    comments: ExternHandler<CommentsHandler>,
    text: ExternHandler<TextHandler>,
    end: ExternHandler<DocumentEndHandler>,
}

impl ExternDocumentContentHandlers {
    #[must_use]
    pub fn as_safe_document_content_handlers(&self) -> DocumentContentHandlers<'_> {
        let mut handlers = DocumentContentHandlers::default();

        add_handler!(handlers, Doctype, self.doctype);
        add_handler!(handlers, Comment, self.comments);
        add_handler!(handlers, TextChunk, self.text);
        add_handler!(handlers, DocumentEnd, self.end);

        handlers
    }
}

pub struct ExternElementContentHandlers {
    element: ExternHandler<ElementHandler>,
    comments: ExternHandler<CommentsHandler>,
    text: ExternHandler<TextHandler>,
}

impl ExternElementContentHandlers {
    #[must_use]
    pub fn as_safe_element_content_handlers(&self) -> ElementContentHandlers<'_> {
        let mut handlers = ElementContentHandlers::default();

        add_handler!(handlers, Element, self.element);
        add_handler!(handlers, Comment, self.comments);
        add_handler!(handlers, TextChunk, self.text);

        handlers
    }
}

pub struct SafeContentHandlers<'b> {
    pub document: Vec<DocumentContentHandlers<'b>>,
    pub element: Vec<(Cow<'b, Selector>, ElementContentHandlers<'b>)>,
}

#[derive(Default)]
pub struct HtmlRewriterBuilder {
    document_content_handlers: Vec<ExternDocumentContentHandlers>,
    element_content_handlers: Vec<(&'static Selector, ExternElementContentHandlers)>,
}

impl HtmlRewriterBuilder {
    #[must_use]
    pub fn get_safe_handlers(&self) -> SafeContentHandlers<'_> {
        SafeContentHandlers {
            document: self
                .document_content_handlers
                .iter()
                .map(|h| h.as_safe_document_content_handlers())
                .collect(),
            element: self
                .element_content_handlers
                .iter()
                .map(|(s, h)| (Cow::Borrowed(*s), h.as_safe_element_content_handlers()))
                .collect(),
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lol_html_rewriter_builder_new() -> *mut HtmlRewriterBuilder {
    to_ptr_mut(HtmlRewriterBuilder::default())
}

#[no_mangle]
pub unsafe extern "C" fn lol_html_rewriter_builder_add_document_content_handlers(
    builder: *mut HtmlRewriterBuilder,
    doctype_handler: Option<DoctypeHandler>,
    doctype_handler_user_data: *mut c_void,
    comments_handler: Option<CommentsHandler>,
    comments_handler_user_data: *mut c_void,
    text_handler: Option<TextHandler>,
    text_handler_user_data: *mut c_void,
    document_end_handler: Option<DocumentEndHandler>,
    document_end_handler_user_data: *mut c_void,
) {
    let builder = to_ref_mut!(builder);

    let handlers = ExternDocumentContentHandlers {
        doctype: ExternHandler::new(doctype_handler, doctype_handler_user_data),
        comments: ExternHandler::new(comments_handler, comments_handler_user_data),
        text: ExternHandler::new(text_handler, text_handler_user_data),
        end: ExternHandler::new(document_end_handler, document_end_handler_user_data),
    };

    builder.document_content_handlers.push(handlers);
}

#[no_mangle]
pub unsafe extern "C" fn lol_html_rewriter_builder_add_element_content_handlers(
    builder: *mut HtmlRewriterBuilder,
    selector: *const Selector,
    element_handler: Option<ElementHandler>,
    element_handler_user_data: *mut c_void,
    comments_handler: Option<CommentsHandler>,
    comments_handler_user_data: *mut c_void,
    text_handler: Option<TextHandler>,
    text_handler_user_data: *mut c_void,
) -> c_int {
    let selector = to_ref!(selector);
    let builder = to_ref_mut!(builder);

    let handlers = ExternElementContentHandlers {
        element: ExternHandler::new(element_handler, element_handler_user_data),
        comments: ExternHandler::new(comments_handler, comments_handler_user_data),
        text: ExternHandler::new(text_handler, text_handler_user_data),
    };

    builder.element_content_handlers.push((selector, handlers));

    0
}

#[no_mangle]
pub unsafe extern "C" fn lol_html_rewriter_builder_free(builder: *mut HtmlRewriterBuilder) {
    drop(to_box!(builder));
}
