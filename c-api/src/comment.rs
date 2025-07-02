use super::*;

#[no_mangle]
pub unsafe extern "C" fn lol_html_comment_text_get(comment: *const Comment) -> Str {
    Str::new(to_ref!(comment).text())
}

#[no_mangle]
pub unsafe extern "C" fn lol_html_comment_text_set(
    comment: *mut Comment,
    text: *const c_char,
    text_len: size_t,
) -> c_int {
    let comment = to_ref_mut!(comment);
    let text = unwrap_or_ret_err_code! { to_str!(text, text_len) };

    unwrap_or_ret_err_code! { comment.set_text(text) };

    0
}

impl_content_mutation_handlers! { comment: Comment [
    /// Inserts the content string before the comment either as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_comment_before => before,
    /// Inserts the content string after the comment either as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_comment_after => after,
    /// Replace the comment with the content of the string which is interpreted
    /// either as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_comment_replace => replace,
    /// Removes the comment.
    @VOID lol_html_comment_remove => remove,
    /// Returns `true` if the comment has been removed.
    @BOOL lol_html_comment_is_removed => removed,
    @STREAM lol_html_comment_streaming_before => streaming_before,
    @STREAM lol_html_comment_streaming_after => streaming_after,
    @STREAM lol_html_comment_streaming_replace => streaming_replace,
    lol_html_comment_source_location_bytes => source_location_bytes,
] }

/// Attaches custom user data to the comment.
///
/// The same comment can be passed to multiple handlers if it has been
/// captured by multiple selectors. It might be handy to store some
/// processing state on the comment, so it can be shared between handlers.
#[no_mangle]
pub unsafe extern "C" fn lol_html_comment_user_data_set(
    comment: *mut Comment,
    user_data: *mut c_void,
) {
    to_ref_mut!(comment).set_user_data(user_data);
}

#[no_mangle]
pub unsafe extern "C" fn lol_html_comment_user_data_get(comment: *const Comment) -> *mut c_void {
    get_user_data!(comment)
}
