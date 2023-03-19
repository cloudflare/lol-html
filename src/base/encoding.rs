use crate::rewriter::AsciiCompatibleEncoding;
use encoding_rs::Encoding;
use std::cell::Cell;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone)]
pub struct SharedEncoding {
    encoding: Rc<Cell<AsciiCompatibleEncoding>>,
}

impl SharedEncoding {
    pub fn new(encoding: AsciiCompatibleEncoding) -> SharedEncoding {
        SharedEncoding {
            encoding: Rc::new(Cell::new(encoding)),
        }
    }

    pub fn get(&self) -> &'static Encoding {
        self.encoding.get().into()
    }

    pub fn set(&self, encoding: AsciiCompatibleEncoding) {
        self.encoding.set(encoding);
    }
}

impl Deref for SharedEncoding {
    type Target = Encoding;

    fn deref(&self) -> &'static Encoding {
        self.get()
    }
}
