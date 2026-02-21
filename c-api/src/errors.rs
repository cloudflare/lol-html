use super::*;

thread_local! {
    pub static LAST_ERROR: RefCell<Option<Box<str>>> = const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "C" fn lol_html_take_last_error() -> Str {
    Str::from_opt(
        LAST_ERROR
            .try_with(|e| e.try_borrow_mut().ok()?.take())
            .ok()
            .flatten(),
    )
}

#[cold]
#[inline(never)]
pub(crate) fn save_last_error(err: String) {
    let err = Some(err.into_boxed_str());
    let _ = crate::errors::LAST_ERROR.try_with(|e| e.try_borrow_mut().map(|mut v| *v = err));
}

#[derive(Error, Debug, Eq, PartialEq, Copy, Clone)]
pub enum CStreamingHandlerError {
    #[error("Not all fields of the struct were initialized")]
    Uninitialized,

    #[error("write_all_callback reported error: {0}")]
    HandlerError(c_int),
}
