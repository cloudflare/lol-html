use super::*;

#[repr(C)]
pub struct TextChunkContent {
    data: *const c_char,
    len: size_t,
}

impl TextChunkContent {
    fn new(chunk: &TextChunk) -> Self {
        let content = chunk.as_str();

        TextChunkContent {
            data: content.as_ptr() as *const c_char,
            len: content.len(),
        }
    }
}

#[no_mangle]
pub extern "C" fn cool_thing_text_chunk_content_get(chunk: *mut TextChunk) -> TextChunkContent {
    TextChunkContent::new(to_ref!(chunk))
}

#[no_mangle]
pub extern "C" fn cool_thing_text_chunk_is_last_in_text_node(chunk: *mut TextChunk) -> bool {
    to_ref!(chunk).last_in_text_node()
}

#[no_mangle]
pub extern "C" fn cool_thing_text_chunk_before(
    chunk: *mut TextChunk,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { chunk.before(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_text_chunk_after(
    chunk: *mut TextChunk,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { chunk.after(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_text_chunk_replace(
    chunk: *mut TextChunk,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { chunk.replace(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_text_chunk_remove(chunk: *mut TextChunk) {
    to_ref_mut!(chunk).remove();
}

#[no_mangle]
pub extern "C" fn cool_thing_text_chunk_is_removed(chunk: *const TextChunk) -> bool {
    to_ref!(chunk).removed()
}

#[no_mangle]
pub extern "C" fn cool_thing_text_chunk_user_data_set(
    chunk: *mut TextChunk,
    user_data: *mut c_void,
) {
    to_ref_mut!(chunk).set_user_data(user_data);
}

#[no_mangle]
pub extern "C" fn cool_thing_text_chunk_user_data_get(chunk: *const TextChunk) -> *mut c_void {
    get_user_data!(chunk)
}
