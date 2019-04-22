use crate::base::Bytes;
use crate::rewritable_units::{Serialize, Token};
use encoding_rs::Encoding;
use std::fmt::{self, Debug};

pub struct Doctype<'i> {
    name: Option<Bytes<'i>>,
    public_id: Option<Bytes<'i>>,
    system_id: Option<Bytes<'i>>,
    force_quirks: bool,
    raw: Bytes<'i>,
    encoding: &'static Encoding,
}

impl<'i> Doctype<'i> {
    pub(super) fn new_token(
        name: Option<Bytes<'i>>,
        public_id: Option<Bytes<'i>>,
        system_id: Option<Bytes<'i>>,
        force_quirks: bool,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Token<'i> {
        Token::Doctype(Doctype {
            name,
            public_id,
            system_id,
            force_quirks,
            raw,
            encoding,
        })
    }

    #[inline]
    pub fn name(&self) -> Option<String> {
        self.name
            .as_ref()
            .map(|n| n.as_lowercase_string(self.encoding))
    }

    #[inline]
    pub fn public_id(&self) -> Option<String> {
        self.public_id.as_ref().map(|i| i.as_string(self.encoding))
    }

    #[inline]
    pub fn system_id(&self) -> Option<String> {
        self.system_id.as_ref().map(|i| i.as_string(self.encoding))
    }

    #[inline]
    #[cfg(feature = "test_api")]
    pub fn force_quirks(&self) -> bool {
        self.force_quirks
    }
}

impl Serialize for Doctype<'_> {
    #[inline]
    fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8])) {
        output_handler(&self.raw);
    }
}

impl Debug for Doctype<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Doctype")
            .field("name", &self.name())
            .field("public_id", &self.public_id())
            .field("system_id", &self.system_id())
            .field("force_quirks", &self.force_quirks)
            .finish()
    }
}
