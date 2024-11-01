use super::*;
use std::error::Error;

thread_local! {
    pub static LAST_ERROR: RefCell<Option<Box<dyn Error>>> = RefCell::new(None);
}

#[no_mangle]
pub extern "C" fn lol_html_take_last_error() -> Str {
    let err = LAST_ERROR.with(|e| e.borrow_mut().take());

    Str::from_opt(err.map(|e| e.to_string()))
}

#[derive(Error, Debug, Eq, PartialEq, Copy, Clone)]
pub enum CStreamingHandlerError {
    #[error("Not all fields of the struct were initialized")]
    Uninitialized,

    #[error("write_all_callback reported error: {0}")]
    HandlerError(c_int),
}
