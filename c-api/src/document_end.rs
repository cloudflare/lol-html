use super::*;

#[no_mangle]
pub extern "C" fn lol_html_doc_end_append(
    document_end: *mut DocumentEnd,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { document_end.append(content, content_len, is_html) }
}
