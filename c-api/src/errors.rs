use super::*;
use failure::Error;

thread_local! {
    pub static LAST_ERROR: RefCell<Option<Error>> = RefCell::new(None);
}

#[no_mangle]
pub extern "C" fn cool_thing_take_last_error() -> *const Str {
    let err = LAST_ERROR.with(|e| e.borrow_mut().take());

    Str::opt_ptr(err.map(|e| e.to_string()))
}

#[no_mangle]
pub extern "C" fn cool_thing_content_handler_error_new(
    msg: *const c_char,
    msg_len: size_t,
) -> *const Error {
    let msg = to_str!(msg, msg_len).unwrap().to_string();

    to_ptr(failure::err_msg(msg))
}
