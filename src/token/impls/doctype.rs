use crate::base::Bytes;
use crate::token::Serialize;
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
    pub(in crate::token) fn new_parsed(
        name: Option<Bytes<'i>>,
        public_id: Option<Bytes<'i>>,
        system_id: Option<Bytes<'i>>,
        force_quirks: bool,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Self {
        Doctype {
            name,
            public_id,
            system_id,
            force_quirks,
            raw,
            encoding,
        }
    }

    #[inline]
    pub fn name(&self) -> Option<String> {
        self.name.as_ref().map(|n| {
            let mut name = n.as_string(self.encoding);

            name.make_ascii_lowercase();

            name
        })
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
    pub fn force_quirks(&self) -> bool {
        self.force_quirks
    }

    // NOTE: not a trait implementation due to the `Borrow` constraint for
    // the `Owned` associated type.
    // See: https://github.com/rust-lang/rust/issues/44950
    #[inline]
    pub fn to_owned(&self) -> Doctype<'static> {
        Doctype {
            name: Bytes::opt_to_owned(&self.name),
            public_id: Bytes::opt_to_owned(&self.public_id),
            system_id: Bytes::opt_to_owned(&self.system_id),
            force_quirks: self.force_quirks,
            raw: self.raw.to_owned(),
            encoding: self.encoding,
        }
    }
}

impl Serialize for Doctype<'_> {
    #[inline]
    fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8])) {
        output_handler(&self.raw);
    }
}

impl Debug for Doctype<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Doctype")
            .field("name", &self.name())
            .field("public_id", &self.public_id())
            .field("system_id", &self.system_id())
            .field("force_quirks", &self.force_quirks)
            .finish()
    }
}
