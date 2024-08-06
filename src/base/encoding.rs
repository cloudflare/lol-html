use crate::rewriter::AsciiCompatibleEncoding;
use encoding_rs::Encoding;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

/// A charset encoding that can be shared and modified.
///
/// This is, for instance, used to adapt the charset dynamically in a [crate::HtmlRewriter] if it
/// encounters a `meta` tag that specifies the charset (that behavior is dependent on
/// [crate::Settings::adjust_charset_on_meta_tag]).
#[derive(Clone)]
pub struct SharedEncoding {
    encoding: Arc<Mutex<AsciiCompatibleEncoding>>,
}

impl SharedEncoding {
    pub fn new(encoding: AsciiCompatibleEncoding) -> SharedEncoding {
        SharedEncoding {
            encoding: Arc::new(Mutex::new(encoding)),
        }
    }

    pub fn get(&self) -> &'static Encoding {
        (*self.encoding.lock().unwrap().deref()).into()
    }

    pub fn set(&self, encoding: AsciiCompatibleEncoding) {
        *self.encoding.lock().unwrap() = encoding;
    }
}

impl Deref for SharedEncoding {
    type Target = Encoding;

    fn deref(&self) -> &'static Encoding {
        self.get()
    }
}
