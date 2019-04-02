use super::LocalNameHash;
use crate::base::{Bytes, Chunk, Range};
use std::fmt::{self, Display, Formatter};

/// Name is a unified DOM string representation that can come either from
/// the selector matching engine or from the parser. Used e.g. for the comparison
/// of attribute names and values.
#[derive(Clone, Debug)]
pub enum Name<'n> {
    Bytes(Bytes<'n>),
    Unencoded(&'n str),
}

impl<'n> Name<'n> {
    #[inline]
    pub fn new(input: &'n Chunk<'n>, range: Range) -> Self {
        Name::Bytes(input.slice(range))
    }
}

impl<'s> From<&'s str> for Name<'s> {
    #[inline]
    fn from(string: &'s str) -> Self {
        Name::Unencoded(string)
    }
}

impl Default for Name<'_> {
    #[inline]
    fn default() -> Self {
        Name::Bytes(Bytes::empty())
    }
}

impl Display for Name<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl PartialEq for Name<'_> {
    #[inline]
    fn eq(&self, other: &Name<'_>) -> bool {
        match (self, other) {
            (Name::Bytes(n1), Name::Bytes(n2)) => n1 == n2,
            _ => unreachable!("Unencoded Names shouldn't be used for the comparison"),
        }
    }
}

impl Eq for Name<'_> {}

/// LocalName is used for the comparison of tag names.
/// In the majority of cases it will be represented as a hash, however for long
/// non-standard tag names it fallsback to the Name representation.
#[derive(Clone, PartialEq, Debug)]
pub enum LocalName<'n> {
    Hash(LocalNameHash),
    Name(Name<'n>),
}

impl<'n> LocalName<'n> {
    #[inline]
    pub fn new(input: &'n Chunk<'n>, range: Range, hash: LocalNameHash) -> Self {
        if hash.is_empty() {
            LocalName::Name(Name::new(input, range))
        } else {
            LocalName::Hash(hash)
        }
    }
}

impl<'s> From<&'s str> for LocalName<'s> {
    #[inline]
    fn from(string: &'s str) -> Self {
        let hash = LocalNameHash::from(string);

        if hash.is_empty() {
            LocalName::Name(string.into())
        } else {
            LocalName::Hash(hash)
        }
    }
}

impl Default for LocalName<'_> {
    #[inline]
    fn default() -> Self {
        LocalName::Hash(LocalNameHash::default())
    }
}

impl Display for LocalName<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Eq for LocalName<'_> {}
