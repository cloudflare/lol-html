#[macro_use]
mod mutations;

mod element;
mod tokens;

use std::any::Any;

pub use self::element::*;
pub use self::mutations::{ContentType, Mutations};
pub use self::tokens::*;

pub trait UserData {
    fn user_data(&self) -> Option<&dyn Any>;
    fn set_user_data(&mut self, data: impl Any);
}
