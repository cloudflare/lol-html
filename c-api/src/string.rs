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
        Str {
            len: string.len(),
            data: Box::into_raw(string.into_boxed_str()) as *const c_char,
        }
    }

    #[inline]
    pub fn opt_ptr(string: Option<String>) -> *const Self {
        match string {
            Some(string) => to_ptr(Self::new(string)),
            None => ptr::null(),
        }
    }
}

impl Drop for Str {
    fn drop(&mut self) {
        let bytes = unsafe { slice::from_raw_parts_mut(self.data as *mut c_char, self.len) };

        drop(unsafe { Box::from_raw(bytes) });
    }
}

#[no_mangle]
pub extern "C" fn lol_html_str_free(string: Str) {
    drop(string);
}
