#[macro_use]
mod tag;

mod local_name_hash;
mod name;
mod text_type;

pub use self::local_name_hash::LocalNameHash;
pub use self::name::LocalName;
pub use self::tag::Tag;
pub use self::text_type::TextType;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Namespace {
    Html,
    Svg,
    MathML,
}

impl Default for Namespace {
    #[inline]
    fn default() -> Self {
        Namespace::Html
    }
}
