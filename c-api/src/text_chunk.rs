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

/// Returns a fat pointer to the UTF8 representation of content of the chunk.
///
/// If the chunk is last in the current text node then content can be an empty string.
///
/// WARNING: The pointer is valid only during the handler execution and
/// should never be leaked outside of handlers.
#[no_mangle]
pub unsafe extern "C" fn lol_html_text_chunk_content_get(
    chunk: *mut TextChunk,
) -> TextChunkContent {
    TextChunkContent::new(to_ref!(chunk))
}

impl_content_mutation_handlers! { text_chunk: TextChunk [
    /// Inserts the content string before the text chunk either as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_text_chunk_before => before,
    /// Inserts the content string after the text chunk either as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_text_chunk_after => after,
    /// Replace the text chunk with the content of the string which is interpreted
    /// either as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_text_chunk_replace => replace,
    /// Removes the text chunk.
    @VOID lol_html_text_chunk_remove => remove,
    /// Returns `true` if the text chunk has been removed.
    @BOOL lol_html_text_chunk_is_removed => removed,
    /// Returns `true` if the chunk is last in the current text node.
    @BOOL lol_html_text_chunk_is_last_in_text_node => last_in_text_node,
    @STREAM lol_html_text_chunk_streaming_before => streaming_before,
    @STREAM lol_html_text_chunk_streaming_after => streaming_after,
    @STREAM lol_html_text_chunk_streaming_replace => streaming_replace,
    lol_html_text_chunk_source_location_bytes => source_location_bytes,
] }

/// Attaches custom user data to the text chunk.
///
/// The same text chunk can be passed to multiple handlers if it has been
/// captured by multiple selectors. It might be handy to store some processing
/// state on the chunk, so it can be shared between handlers.
#[no_mangle]
pub unsafe extern "C" fn lol_html_text_chunk_user_data_set(
    chunk: *mut TextChunk,
    user_data: *mut c_void,
) {
    to_ref_mut!(chunk).set_user_data(user_data);
}

/// Returns user data attached to the text chunk.
#[no_mangle]
pub unsafe extern "C" fn lol_html_text_chunk_user_data_get(chunk: *const TextChunk) -> *mut c_void {
    get_user_data!(chunk)
}
