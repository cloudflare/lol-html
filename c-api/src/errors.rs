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
