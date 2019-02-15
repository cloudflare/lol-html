mod lexeme;
mod tag_name_info;
mod token_outline;

pub use self::lexeme::Lexeme;
pub use self::tag_name_info::TagNameInfo;
pub use self::token_outline::{AttributeOultine, TokenOutline};

#[derive(Debug)]
pub enum TagHint<'i> {
    StartTag(TagNameInfo<'i>),
    EndTag(TagNameInfo<'i>),
}
