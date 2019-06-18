use super::*;

// NOTE: we don't use CStr and CString as the transfer type because UTF8
// string comming from both sides can contain interior NULLs.
#[repr(C)]
pub struct Str {
    data: *const c_char,
    len: size_t,
}

impl Str {
    pub fn new(string: String) -> Self {
        let len = string.len();
        let bytes = string.into_boxed_str().into_boxed_bytes();

        Str {
            data: Box::into_raw(bytes) as *const c_char,
            len,
        }
    }

    #[inline]
    pub fn ptr(string: String) -> *const Self {
        to_ptr(Self::new(string))
    }

    #[inline]
    pub fn opt_ptr(string: Option<String>) -> *const Self {
        match string {
            Some(string) => Self::ptr(string),
            None => ptr::null(),
        }
    }
}

impl Drop for Str {
    fn drop(&mut self) {
        // NOTE: empty string buffer is essentialy a Unique::empty()
        // which is a NonZero pointer with a phatom data attached.
        // And NonZero is just a dangling non-zero pointer. So,
        // attempt of freeing it will result in a segfault.
        if self.len > 0 {
            let string_data = self.data as *mut c_char;

            drop(to_box!(string_data));
        }
    }
}

#[no_mangle]
pub extern "C" fn cool_thing_str_free(string: Str) {
    drop(string);
}
