use std::any::Any;

pub use self::element::*;
pub use self::mutations::{ContentType, Mutations};
pub use self::tokens::*;

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
    use crate::test_utils::Output;
    use crate::*;
    use encoding_rs::Encoding;
    use std::convert::TryFrom;

    pub fn rewrite_html(
        html: &str,
        encoding: &'static Encoding,
        element_content_handlers: Vec<(&Selector, ElementContentHandlers)>,
        document_content_handlers: Vec<DocumentContentHandlers>,
    ) -> String {
        let mut output = Output::new(encoding);

        {
            let mut rewriter = HtmlRewriter::try_from(Settings {
                element_content_handlers,
                document_content_handlers,
                encoding: encoding.name(),
                buffer_capacity: 2048,
                output_sink: |c: &[u8]| output.push(c),
            })
            .unwrap();

            rewriter.write(html.as_bytes()).unwrap();
            rewriter.end().unwrap();
        }

        output.into()
    }
}
