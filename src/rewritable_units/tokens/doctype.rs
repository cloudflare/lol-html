use crate::base::Bytes;
use crate::rewritable_units::{Serialize, Token};
use encoding_rs::Encoding;
use std::any::Any;
use std::fmt::{self, Debug};

pub struct Doctype<'i> {
    name: Option<Bytes<'i>>,
    public_id: Option<Bytes<'i>>,
    system_id: Option<Bytes<'i>>,
    force_quirks: bool,
    raw: Bytes<'i>,
    encoding: &'static Encoding,
    user_data: Box<dyn Any>,
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
            user_data: Box::new(()),
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
    #[cfg(feature = "integration_test")]
    pub fn force_quirks(&self) -> bool {
        self.force_quirks
    }
}

impl_user_data!(Doctype<'_>);

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

#[cfg(test)]
mod tests {
    use crate::rewritable_units::test_utils::*;
    use crate::*;
    use encoding_rs::{Encoding, UTF_8};

    fn rewrite_doctype(
        html: &[u8],
        encoding: &'static Encoding,
        mut handler: impl FnMut(&mut Doctype),
    ) -> String {
        let mut handler_called = false;

        let output = rewrite_html(
            html,
            encoding,
            vec![],
            vec![DocumentContentHandlers::default().doctype(|c| {
                handler_called = true;
                handler(c);
                Ok(())
            })],
        );

        assert!(handler_called);

        output
    }

    #[test]
    fn user_data() {
        rewrite_doctype(b"<!doctype>", UTF_8, |d| {
            d.set_user_data(42usize);

            assert_eq!(*d.user_data().downcast_ref::<usize>().unwrap(), 42usize);

            *d.user_data_mut().downcast_mut::<usize>().unwrap() = 1337usize;

            assert_eq!(*d.user_data().downcast_ref::<usize>().unwrap(), 1337usize);
        });
    }

    #[test]
    fn serialization() {
        for (html, enc) in encoded(r#"<!DOCTYPE html SYSTEM "Ĥey">"#) {
            let output = rewrite_doctype(&html, enc, |_| {});

            assert_eq!(output, r#"<!DOCTYPE html SYSTEM "Ĥey">"#);
        }
    }
}
