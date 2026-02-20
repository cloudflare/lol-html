use super::*;

// NOTE: we don't use CStr and CString as the transfer type because UTF8
// string comming from both sides can contain interior NULLs.
#[repr(C)]
pub struct Str {
    data: *const c_char,
    len: size_t,
}

impl Str {
    pub const EMPTY: Self = Self {
        data: std::ptr::null(),
        len: 0,
    };

    #[must_use]
    pub fn new(string: impl Into<Box<str>>) -> Self {
        let string = string.into();
        Self {
            len: string.len(),
            data: Box::into_raw(string).cast::<c_char>(),
        }
    }

    /// Convert an `Option<String>` to a C-style string.
    ///
    /// If `string` is `None`, `data` will be set to `NULL`.
    #[inline]
    #[must_use]
    pub fn from_opt(string: Option<impl Into<Box<str>>>) -> Self {
        match string {
            Some(string) => Self::new(string),
            None => Self {
                data: ptr::null(),
                len: 0,
            },
        }
    }
}

impl Drop for Str {
    fn drop(&mut self) {
        if self.data.is_null() {
            return;
        }
        let bytes = unsafe { slice::from_raw_parts_mut(self.data.cast_mut(), self.len) };

        drop(unsafe { Box::from_raw(bytes) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_str_free(string: Str) {
    drop(string);
}
