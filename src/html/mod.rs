#[macro_use]
mod tag;

mod local_name;
mod local_name_hash;
mod text_type;

pub use self::local_name::LocalName;
pub use self::local_name_hash::LocalNameHash;
pub use self::tag::Tag;
pub use self::text_type::TextType;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Namespace {
    Html,
    Svg,
    MathML,
}
