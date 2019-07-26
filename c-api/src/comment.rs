use super::*;

#[no_mangle]
pub extern "C" fn cool_thing_comment_text_get(comment: *const Comment) -> Str {
    Str::new(to_ref!(comment).text())
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_text_set(
    comment: *mut Comment,
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
    comment: *mut Comment,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { comment.before(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_after(
    comment: *mut Comment,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { comment.after(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_replace(
    comment: *mut Comment,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { comment.replace(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_remove(comment: *mut Comment) {
    to_ref_mut!(comment).remove();
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_is_removed(comment: *const Comment) -> bool {
    to_ref!(comment).removed()
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_user_data_set(comment: *mut Comment, user_data: *mut c_void) {
    to_ref_mut!(comment).set_user_data(user_data);
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_user_data_get(comment: *const Comment) -> *mut c_void {
    get_user_data!(comment)
}
