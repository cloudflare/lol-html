use super::*;

#[repr(C)]
pub struct TextChunkContent {
    data: *const c_char,
    len: size_t,
}

impl TextChunkContent {
    fn new(chunk: &TextChunk) -> Self {
        let content = chunk.as_str();

        Self {
            data: content.as_ptr().cast::<c_char>(),
            len: content.len(),
        }
    }
}

#[no_mangle]
pub extern "C" fn lol_html_text_chunk_content_get(chunk: *mut TextChunk) -> TextChunkContent {
    TextChunkContent::new(to_ref!(chunk))
}

impl_content_mutation_handlers! { text_chunk: TextChunk [
    lol_html_text_chunk_before => before,
    lol_html_text_chunk_after => after,
    lol_html_text_chunk_replace => replace,
    @VOID lol_html_text_chunk_remove => remove,
    @BOOL lol_html_text_chunk_is_removed => removed,
    @BOOL lol_html_text_chunk_is_last_in_text_node => last_in_text_node,
    @STREAM lol_html_text_chunk_streaming_before => streaming_before,
    @STREAM lol_html_text_chunk_streaming_after => streaming_after,
    @STREAM lol_html_text_chunk_streaming_replace => streaming_replace,
] }

#[no_mangle]
pub extern "C" fn lol_html_text_chunk_user_data_set(chunk: *mut TextChunk, user_data: *mut c_void) {
    to_ref_mut!(chunk).set_user_data(user_data);
}

#[no_mangle]
pub extern "C" fn lol_html_text_chunk_user_data_get(chunk: *const TextChunk) -> *mut c_void {
    get_user_data!(chunk)
}
