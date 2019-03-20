use super::*;

#[no_mangle]
pub extern "C" fn cool_thing_comment_text_get(comment: *const Comment<'_>) -> Str {
    Str::new(to_ref!(comment).text())
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_text_set(
    comment: *mut Comment<'_>,
    text: *const c_char,
    text_len: size_t,
) -> c_int {
    let comment = to_ref_mut!(comment);
    let text = unwrap_or_ret_err_code! { to_str!(text, text_len) };

    unwrap_or_ret_err_code! { comment.set_text(text) };

    0
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_before(
    comment: *mut Comment<'_>,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { comment.before(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_after(
    comment: *mut Comment<'_>,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { comment.after(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_replace(
    comment: *mut Comment<'_>,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { comment.replace(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_remove(comment: *mut Comment<'_>) {
    to_ref_mut!(comment).remove();
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_is_removed(comment: *const Comment<'_>) -> bool {
    to_ref!(comment).removed()
}
