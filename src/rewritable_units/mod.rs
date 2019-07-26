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
