use super::*;

#[no_mangle]
pub extern "C" fn cool_thing_str_free(string: Str) {
    // NOTE: empty string buffer is essentialy a Unique::empty()
    // which is a NonZero pointer with a phatom data attached.
    // And NonZero is just a dangling non-zero pointer. So,
    // attempt of freeing it will result in a segfault.
    if string.len > 0 {
        let string_data = string.data as *mut c_char;

        drop(to_box!(string_data));
    }
}

#[no_mangle]
pub extern "C" fn cool_thing_take_last_error() -> *const Str {
    let err = LAST_ERROR.with(|e| e.borrow_mut().take());

    Str::opt_ptr(err.map(|e| e.to_string()))
}
