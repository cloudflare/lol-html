use super::*;
use libc::c_void;

macro_rules! wrap_handler {
    ($handler:expr, $user_data:expr) => {
        move |arg: &mut _| unsafe { $handler(arg, $user_data) }
    };
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_builder_new() -> *mut HtmlRewriterBuilder<'static> {
    to_ptr_mut(HtmlRewriterBuilder::default())
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_builder_add_document_content_handlers(
    builder: *mut HtmlRewriterBuilder<'static>,
    doctype_handler: Option<unsafe extern "C" fn(*mut Doctype, *mut c_void)>,
    doctype_handler_user_data: *mut c_void,
    comments_handler: Option<unsafe extern "C" fn(*mut Comment, *mut c_void)>,
    comments_handler_user_data: *mut c_void,
    text_handler: Option<unsafe extern "C" fn(*mut TextChunk, *mut c_void)>,
    text_handler_user_data: *mut c_void,
) {
    let builder = to_ref_mut!(builder);
    let mut handlers = DocumentContentHandlers::default();

    if let Some(handler) = doctype_handler {
        handlers = handlers.doctype(wrap_handler!(handler, doctype_handler_user_data));
    }

    if let Some(handler) = comments_handler {
        handlers = handlers.comments(wrap_handler!(handler, comments_handler_user_data));
    }

    if let Some(handler) = text_handler {
        handlers = handlers.text(wrap_handler!(handler, text_handler_user_data));
    }

    builder.on_document(handlers);
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_builder_add_element_content_handlers(
    builder: *mut HtmlRewriterBuilder<'static>,
    selector: *const c_char,
    selector_len: size_t,
    element_handler: Option<unsafe extern "C" fn(*mut Element, *mut c_void)>,
    element_handler_user_data: *mut c_void,
    comments_handler: Option<unsafe extern "C" fn(*mut Comment, *mut c_void)>,
    comments_handler_user_data: *mut c_void,
    text_handler: Option<unsafe extern "C" fn(*mut TextChunk, *mut c_void)>,
    text_handler_user_data: *mut c_void,
) -> c_int {
    let selector = unwrap_or_ret_err_code! { to_str!(selector, selector_len) };
    let builder = to_ref_mut!(builder);
    let mut handlers = ElementContentHandlers::default();

    if let Some(handler) = element_handler {
        handlers = handlers.element(wrap_handler!(handler, element_handler_user_data));
    }

    if let Some(handler) = comments_handler {
        handlers = handlers.comments(wrap_handler!(handler, comments_handler_user_data));
    }

    if let Some(handler) = text_handler {
        handlers = handlers.text(wrap_handler!(handler, text_handler_user_data));
    }

    unwrap_or_ret_err_code! { builder.on(selector, handlers) };

    0
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_builder_free(builder: *mut HtmlRewriterBuilder<'static>) {
    drop(to_box!(builder));
}
