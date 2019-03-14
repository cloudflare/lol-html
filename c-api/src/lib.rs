use cool_thing::*;

#[no_mangle]
pub extern fn cool_thing_new_document_content_handlers() -> *mut DocumentContentHandlers<'static> {
    Box::into_raw(Box::new(DocumentContentHandlers::default()))
}
