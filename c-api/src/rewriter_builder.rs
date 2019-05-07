use super::*;

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_builder_new() -> *mut HtmlRewriterBuilder<'static> {
    to_ptr_mut(HtmlRewriterBuilder::default())
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_builder_add_document_content_handlers(
    builder: *mut HtmlRewriterBuilder<'static>,
    doctype_handler: Option<extern "C" fn(*mut Doctype)>,
    comments_handler: Option<extern "C" fn(*mut Comment)>,
    text_handler: Option<extern "C" fn(*mut TextChunk)>,
) {
    let builder = to_ref_mut!(builder);
    let mut handlers = DocumentContentHandlers::default();

    if let Some(handler) = doctype_handler {
        handlers = handlers.doctype(move |d| handler(d));
    }

    if let Some(handler) = comments_handler {
        handlers = handlers.comments(move |c| handler(c));
    }

    if let Some(handler) = text_handler {
        handlers = handlers.text(move |c| handler(c));
    }

    builder.on_document(handlers);
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_builder_add_element_content_handlers(
    builder: *mut HtmlRewriterBuilder<'static>,
    selector: *const c_char,
    selector_len: size_t,
    element_handler: Option<extern "C" fn(*mut Element)>,
    comments_handler: Option<extern "C" fn(*mut Comment)>,
    text_handler: Option<extern "C" fn(*mut TextChunk)>,
) -> c_int {
    let selector = unwrap_or_ret_err_code! { to_str!(selector, selector_len) };
    let builder = to_ref_mut!(builder);
    let mut handlers = ElementContentHandlers::default();

    if let Some(handler) = element_handler {
        handlers = handlers.element(move |e| handler(e));
    }

    if let Some(handler) = comments_handler {
        handlers = handlers.comments(move |c| handler(c));
    }

    if let Some(handler) = text_handler {
        handlers = handlers.text(move |c| handler(c));
    }

    unwrap_or_ret_err_code! { builder.on(selector, handlers) };

    0
}
