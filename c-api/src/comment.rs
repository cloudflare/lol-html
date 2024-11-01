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
    lol_html_comment_before => before,
    lol_html_comment_after => after,
    lol_html_comment_replace => replace,
    @VOID lol_html_comment_remove => remove,
    @BOOL lol_html_comment_is_removed => removed,
    @STREAM lol_html_comment_streaming_before => streaming_before,
    @STREAM lol_html_comment_streaming_after => streaming_after,
    @STREAM lol_html_comment_streaming_replace => streaming_replace,
] }

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
