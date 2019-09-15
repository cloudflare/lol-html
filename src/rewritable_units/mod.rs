use std::any::Any;

pub use self::element::*;
pub use self::mutations::{ContentType, Mutations};
pub use self::tokens::*;

/// TODO docs with examples
pub trait UserData {
    fn user_data(&self) -> &dyn Any;
    fn user_data_mut(&mut self) -> &mut dyn Any;
    fn set_user_data(&mut self, data: impl Any);
}

macro_rules! impl_user_data {
    ($Unit:ident<$($lt:lifetime),+>) => {
        impl crate::rewritable_units::UserData for $Unit<$($lt),+> {
            #[inline]
            fn user_data(&self) -> &dyn Any {
                &*self.user_data
            }

            #[inline]
            fn user_data_mut(&mut self) -> &mut dyn Any {
                &mut *self.user_data
            }

            #[inline]
            fn set_user_data(&mut self, data: impl Any){
                self.user_data = Box::new(data);
            }
        }
    };
}

#[macro_use]
mod mutations;

mod element;
mod tokens;

#[cfg(test)]
mod test_utils {
    use crate::test_utils::{Output, ASCII_COMPATIBLE_ENCODINGS};
    use crate::*;
    use encoding_rs::Encoding;

    pub fn encoded(input: &str) -> Vec<(Vec<u8>, &'static Encoding)> {
        ASCII_COMPATIBLE_ENCODINGS
            .iter()
            .filter_map(|enc| {
                let (input, _, has_unmappable_characters) = enc.encode(input);

                // NOTE: there is no character in existence outside of ASCII range
                // that can be represented in all the ASCII-compatible encodings.
                // So, if test cases contains some non-ASCII characters that can't
                // be represented in the given encoding then we just skip it.
                // It is OK to do so, because our intention is not to test the
                // encoding library itself (it is already well tested), but test
                // how our own code works with non-ASCII characters.
                if has_unmappable_characters {
                    None
                } else {
                    Some((input.into_owned(), *enc))
                }
            })
            .collect()
    }

    pub fn rewrite_html(
        html: &[u8],
        encoding: &'static Encoding,
        element_content_handlers: Vec<(&Selector, ElementContentHandlers)>,
        document_content_handlers: Vec<DocumentContentHandlers>,
    ) -> String {
        let mut output = Output::new(encoding);

        {
            let mut rewriter = HtmlRewriter::try_new(
                Settings {
                    element_content_handlers,
                    document_content_handlers,
                    encoding: encoding.name(),
                    ..Settings::default()
                },
                |c: &[u8]| output.push(c),
            )
            .unwrap();

            rewriter.write(html).unwrap();
            rewriter.end().unwrap();
        }

        output.into()
    }
}
