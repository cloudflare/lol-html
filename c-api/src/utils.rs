use super::*;

#[no_mangle]
pub extern "C" fn cool_thing_str_free(string: Str) {
    let string_data = string.data as *mut c_char;

    drop(to_box!(string_data));
}

#[no_mangle]
pub extern "C" fn cool_thing_take_last_error() -> *const Str {
    let err = LAST_ERROR.with(|e| e.borrow_mut().take());

    Str::opt_ptr(err.map(|e| e.to_string()))
}
